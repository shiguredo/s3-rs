use crate::error::Error;
use crate::request::S3Response;

// -------------------------------------------------------
// バリデーション
// -------------------------------------------------------

/// 必須パラメータのバリデーション
///
/// `None` または空文字列の場合に `Error::InvalidInput` を返す。
pub(crate) fn required<'a>(value: Option<&'a str>, name: &str) -> Result<&'a str, Error> {
    let v = value.ok_or_else(|| Error::InvalidInput(format!("{name} is required")))?;
    if v.is_empty() {
        return Err(Error::InvalidInput(format!("{name} must not be empty")));
    }
    Ok(v)
}

/// Presigned URL の最小有効期限 (秒)
///
/// https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html
const PRESIGN_MIN_EXPIRES_SECS: u64 = 1;
/// Presigned URL の最大有効期限 (秒) — 7 日間
///
/// https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html
const PRESIGN_MAX_EXPIRES_SECS: u64 = 604800;

/// Presigned URL の有効期限を検証する (1 〜 604800 秒)
pub(crate) fn validate_presign_expires(expires_in_secs: u64) -> Result<(), Error> {
    if !(PRESIGN_MIN_EXPIRES_SECS..=PRESIGN_MAX_EXPIRES_SECS).contains(&expires_in_secs) {
        return Err(Error::InvalidInput(format!(
            "expires_in_secs must be between {PRESIGN_MIN_EXPIRES_SECS} and {PRESIGN_MAX_EXPIRES_SECS} (7 days)"
        )));
    }
    Ok(())
}

/// パート番号を検証する (1 〜 10000)
///
/// HeadObject / GetObject / UploadPart / UploadPartCopy で共通して使う。
/// https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadObject.html
pub(crate) fn validate_part_number(part_number: i32) -> Result<(), Error> {
    if !(1..=10000).contains(&part_number) {
        return Err(Error::InvalidInput(
            "part_number must be between 1 and 10000".to_string(),
        ));
    }
    Ok(())
}

// -------------------------------------------------------
// エラー解析
// -------------------------------------------------------

/// 2xx レスポンスのボディに `<Error>` が含まれていないか検査する
///
/// CompleteMultipartUpload と CopyObject は 200 OK でボディにエラーを返すことがある。
/// ボディサイズが 10MB 以下の場合は XML 全体をパースして `<Error>` ルートタグの存在を確認する。
/// 10MB 超の場合は先頭 8KB をスキャンして `<Error` 文字列の有無を確認する。
pub(crate) fn check_body_error(response: &S3Response) -> Result<(), Error> {
    let has_error = if response.body.len() <= MAX_XML_BODY_SIZE {
        let text = std::str::from_utf8(&response.body).unwrap_or("");
        crate::xml::has_error_root(text)
    } else {
        // 10MB 超のボディは先頭 8KB で <Error の存在を簡易スキャンする
        let scan_len = std::cmp::min(response.body.len(), 8192);
        let head = &response.body[..scan_len];
        let head_str = std::str::from_utf8(head).unwrap_or("");
        head_str.contains("<Error") || head_str.contains("<Error ")
    };

    if has_error {
        return Err(parse_error_response_with_status(
            response.status_code,
            &response.body,
        ));
    }
    Ok(())
}

/// XML レスポンスのボディサイズ上限 (10MB)
///
/// xml-rs には入力サイズの制限機能がないため、パース前にサイズチェックを行う。
/// S3 の XML レスポンスは通常数百 KB 以内（ListObjectsV2 の max-keys=1000 でも十分収まる）。
/// 10MB はプロキシ経由での改ざんや予期しない巨大レスポンスに対する防御ライン。
const MAX_XML_BODY_SIZE: usize = 10 * 1024 * 1024;

/// レスポンスボディを XML テキストとしてパースする（サイズチェック付き）
pub(crate) fn xml_body_text(body: &[u8]) -> Result<&str, Error> {
    if body.len() > MAX_XML_BODY_SIZE {
        return Err(Error::InvalidResponse(format!(
            "XML response body too large: {} bytes (max {})",
            body.len(),
            MAX_XML_BODY_SIZE
        )));
    }
    std::str::from_utf8(body)
        .map_err(|_| Error::InvalidResponse("non-UTF-8 response body".to_string()))
}

/// S3 エラーレスポンスを解析する
pub(crate) fn parse_error_response(response: &S3Response) -> Error {
    parse_error_response_with_status(response.status_code, &response.body)
}

/// HEAD レスポンスの失敗時に HTTP ステータスコードからエラーを構築する
///
/// HEAD レスポンスは body を返さないため、XML ベースのエラー解析は不可能。
/// HTTP ステータスコードから推定可能な範囲でエラーコードを設定する。
pub(crate) fn head_error_from_status(status_code: u16) -> Error {
    if status_code == 304 {
        return Error::NotModified;
    }
    if status_code == 412 {
        return Error::PreconditionFailed;
    }
    let (code, message) = match status_code {
        400 => ("BadRequest", "bad request"),
        403 => ("AccessDenied", "access denied"),
        404 => ("NotFound", "not found"),
        405 => ("MethodNotAllowed", "method not allowed"),
        416 => ("InvalidRange", "invalid range"),
        500 => ("InternalError", "internal server error"),
        503 => ("ServiceUnavailable", "service unavailable"),
        _ => ("HttpError", "request failed"),
    };
    Error::S3 {
        status_code,
        code: code.to_string(),
        message: message.to_string(),
    }
}

/// ステータスコードとボディから S3 エラーを構築する
fn parse_error_response_with_status(status_code: u16, body: &[u8]) -> Error {
    let (code, message) = parse_s3_error_xml(body)
        .unwrap_or_else(|_| ("UnknownError".to_string(), "unknown error".to_string()));

    Error::S3 {
        status_code,
        code,
        message,
    }
}

fn parse_s3_error_xml(body: &[u8]) -> Result<(String, String), Error> {
    crate::xml::parse_s3_error(body)
}

// -------------------------------------------------------
// ユーティリティ
// -------------------------------------------------------

pub(crate) fn base64_md5(data: &[u8]) -> String {
    use base64ct::{Base64, Encoding};
    use md5::{Digest, Md5};
    let hash = Md5::digest(data);
    Base64::encode_string(hash.as_slice())
}

/// SSE-C キー (Base64) から MD5 (Base64) を自動計算する
///
/// sse_customer_key が指定されていて sse_customer_key_md5 が未指定の場合に
/// 自動的に MD5 を計算する。
pub(crate) fn compute_sse_c_key_md5(base64_key: &str) -> Result<String, Error> {
    use base64ct::{Base64, Encoding};
    use md5::{Digest, Md5};
    let key_bytes = Base64::decode_vec(base64_key)
        .map_err(|_| Error::InvalidInput("SSE-C key must be valid Base64".to_string()))?;
    let hash = Md5::digest(&key_bytes);
    Ok(Base64::encode_string(hash.as_slice()))
}

/// SSE-C ヘッダーを構築して追加する
///
/// リクエストヘッダーに `x-amz-server-side-encryption-customer-algorithm` /
/// `-key` / `-key-md5` を追加する。`copy_source` が true の場合は
/// `x-amz-copy-source-server-side-encryption-customer-*` を使用する。
/// `computed_key_md5` は呼び出し側で宣言した `Option<String>` の参照で、
/// MD5 計算結果の生存期間を呼び出し側が管理する (追加したヘッダー値の参照が
/// 関数終了後も有効である必要があるため)。
///
/// 仕様: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html
/// > x-amz-server-side-encryption-customer-algorithm: Specifies the algorithm to use when encrypting the object (for example, AES256).
/// > x-amz-server-side-encryption-customer-key: Specifies the customer-provided encryption key for Amazon S3 to use in encrypting data.
/// > x-amz-server-side-encryption-customer-key-MD5: Specifies the 128-bit MD5 digest of the encryption key according to RFC 1321.
///
/// コピー元 SSE-C のヘッダー名は CopyObject API の仕様に基づく。
/// 仕様: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html
/// > x-amz-copy-source-server-side-encryption-customer-algorithm: Specifies the algorithm to use when decrypting the source object (for example, AES256).
/// > x-amz-copy-source-server-side-encryption-customer-key: Specifies the customer-provided encryption key for Amazon S3 to use to decrypt the source object.
/// > x-amz-copy-source-server-side-encryption-customer-key-MD5: Specifies the 128-bit MD5 digest of the encryption key according to RFC 1321.
///
/// ヘッダー名は AWS の仕様変更により将来変わる可能性がある。
pub(crate) fn add_sse_c_headers<'a>(
    extra_headers: &mut Vec<(&'a str, &'a str)>,
    sse_customer_algorithm: Option<&'a str>,
    sse_customer_key: Option<&'a str>,
    computed_key_md5: &'a mut Option<String>,
    copy_source: bool,
) -> Result<(), Error> {
    let (algorithm_header, key_header, key_md5_header) = if copy_source {
        (
            "x-amz-copy-source-server-side-encryption-customer-algorithm",
            "x-amz-copy-source-server-side-encryption-customer-key",
            "x-amz-copy-source-server-side-encryption-customer-key-md5",
        )
    } else {
        (
            "x-amz-server-side-encryption-customer-algorithm",
            "x-amz-server-side-encryption-customer-key",
            "x-amz-server-side-encryption-customer-key-md5",
        )
    };

    if let Some(v) = sse_customer_algorithm {
        extra_headers.push((algorithm_header, v));
    }
    if let Some(v) = sse_customer_key {
        extra_headers.push((key_header, v));
        *computed_key_md5 = Some(compute_sse_c_key_md5(v)?);
        extra_headers.push((
            key_md5_header,
            computed_key_md5
                .as_deref()
                .expect("computed_key_md5 is set just above"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `required()` は Some(非空) をそのまま返す
    #[test]
    fn test_required_some_non_empty() {
        let result = required(Some("hello"), "test_field");
        assert_eq!(result.expect("非空の値が通ること"), "hello");
    }

    /// `required()` は None を Error::InvalidInput として拒否する
    #[test]
    fn test_required_none() {
        let result = required(None, "test_field");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    /// `required()` は空文字列を Error::InvalidInput として拒否する
    #[test]
    fn test_required_empty_string() {
        let result = required(Some(""), "test_field");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    /// `required()` は空白のみの文字列は許容する (trim は行わない)
    #[test]
    fn test_required_whitespace_only() {
        let result = required(Some("   "), "test_field");
        assert_eq!(result.expect("空白のみは空ではないので通ること"), "   ");
    }

    /// `validate_presign_expires()` は最小値 1 と最大値 604800 を受理する
    #[test]
    fn test_validate_presign_expires_boundaries() {
        assert!(validate_presign_expires(1).is_ok());
        assert!(validate_presign_expires(604800).is_ok());
    }

    /// `validate_presign_expires()` は 0 秒と 604801 秒を Error::InvalidInput として拒否する
    #[test]
    fn test_validate_presign_expires_out_of_range() {
        assert!(matches!(
            validate_presign_expires(0),
            Err(Error::InvalidInput(_))
        ));
        assert!(matches!(
            validate_presign_expires(604801),
            Err(Error::InvalidInput(_))
        ));
    }

    /// `validate_part_number()` は最小値 1 と最大値 10000 を受理する
    #[test]
    fn test_validate_part_number_boundaries() {
        assert!(validate_part_number(1).is_ok());
        assert!(validate_part_number(10000).is_ok());
    }

    /// `validate_part_number()` は 0 と 10001 を Error::InvalidInput として拒否する
    #[test]
    fn test_validate_part_number_out_of_range() {
        assert!(matches!(
            validate_part_number(0),
            Err(Error::InvalidInput(_))
        ));
        assert!(matches!(
            validate_part_number(10001),
            Err(Error::InvalidInput(_))
        ));
    }

    /// HEAD レスポンスの 304 (Not Modified) は Error::NotModified を返す
    #[test]
    fn test_head_error_from_status_not_modified() {
        assert!(matches!(head_error_from_status(304), Error::NotModified));
    }

    /// HEAD レスポンスの 412 (Precondition Failed) は Error::PreconditionFailed を返す
    #[test]
    fn test_head_error_from_status_precondition_failed() {
        assert!(matches!(
            head_error_from_status(412),
            Error::PreconditionFailed
        ));
    }

    /// HEAD レスポンスの各ステータスコードが正しいエラーコードに写像される
    #[test]
    fn test_head_error_from_status_mapping() {
        for (status, expected_code) in [
            (400, "BadRequest"),
            (403, "AccessDenied"),
            (404, "NotFound"),
            (405, "MethodNotAllowed"),
            (416, "InvalidRange"),
            (500, "InternalError"),
            (503, "ServiceUnavailable"),
            (599, "HttpError"),
        ] {
            match head_error_from_status(status) {
                Error::S3 { code, .. } => assert_eq!(code, expected_code),
                other => panic!("予期しないエラー: {other:?}"),
            }
        }
    }

    /// `base64_md5()` は入力の MD5 ダイジェストを Base64 エンコードして返す
    #[test]
    fn test_base64_md5() {
        // "abc" の MD5 (RFC 1321 のテストベクトル) を Base64 で表現した値
        assert_eq!(base64_md5(b"abc"), "kAFQmDzST7DWlj99KOF/cg==");
    }

    /// `compute_sse_c_key_md5()` は Base64 キーをデコードして MD5 を再エンコードする
    ///
    /// "0123456789abcdef" (16 バイト) の Base64 表現を入力し、デコードが正しければ
    /// 元バイト列を直接 MD5 化した値と一致することを検証する
    #[test]
    fn test_compute_sse_c_key_md5() {
        let base64_key = "MDEyMzQ1Njc4OWFiY2RlZg==";
        let md5 = compute_sse_c_key_md5(base64_key).expect("有効な Base64 であること");
        // デコードしたキーを MD5 化して再エンコードした値と一致すること
        assert_eq!(md5, base64_md5(b"0123456789abcdef"));
    }

    /// `compute_sse_c_key_md5()` は不正な Base64 を Error::InvalidInput として拒否する
    #[test]
    fn test_compute_sse_c_key_md5_invalid_base64() {
        assert!(matches!(
            compute_sse_c_key_md5("invalid!"),
            Err(Error::InvalidInput(_))
        ));
    }

    /// `add_sse_c_headers()` は標準 SSE-C ヘッダーを algorithm → key → MD5 の順に追加する
    #[test]
    fn test_add_sse_c_headers_standard() {
        let mut headers = Vec::new();
        let mut computed_key_md5 = None;

        add_sse_c_headers(
            &mut headers,
            Some("AES256"),
            Some("MDEyMzQ1Njc4OWFiY2RlZg=="),
            &mut computed_key_md5,
            false,
        )
        .expect("有効なヘッダーであること");

        assert_eq!(headers.len(), 3);
        assert_eq!(
            headers[0],
            ("x-amz-server-side-encryption-customer-algorithm", "AES256")
        );
        assert_eq!(
            headers[1],
            (
                "x-amz-server-side-encryption-customer-key",
                "MDEyMzQ1Njc4OWFiY2RlZg=="
            )
        );
        assert_eq!(
            headers[2].0,
            "x-amz-server-side-encryption-customer-key-md5"
        );
        // MD5 は "0123456789abcdef" の MD5 を Base64 で表現した値と一致すること
        assert_eq!(headers[2].1, "QDKvjWEDUSOQbljgZxQMxQ==");
        // 計算結果が呼び出し側の Option にも保存されること
        assert_eq!(
            computed_key_md5.as_deref(),
            Some("QDKvjWEDUSOQbljgZxQMxQ==")
        );
    }

    /// `add_sse_c_headers()` は copy_source 指定時にコピー元用ヘッダーを使う
    #[test]
    fn test_add_sse_c_headers_copy_source() {
        let mut headers = Vec::new();
        let mut computed_key_md5 = None;

        add_sse_c_headers(
            &mut headers,
            Some("AES256"),
            Some("MDEyMzQ1Njc4OWFiY2RlZg=="),
            &mut computed_key_md5,
            true,
        )
        .expect("有効なヘッダーであること");

        assert_eq!(headers.len(), 3);
        assert_eq!(
            headers[0],
            (
                "x-amz-copy-source-server-side-encryption-customer-algorithm",
                "AES256"
            )
        );
        assert_eq!(
            headers[1],
            (
                "x-amz-copy-source-server-side-encryption-customer-key",
                "MDEyMzQ1Njc4OWFiY2RlZg=="
            )
        );
        assert_eq!(
            headers[2].0,
            "x-amz-copy-source-server-side-encryption-customer-key-md5"
        );
        // MD5 は "0123456789abcdef" の MD5 を Base64 で表現した値と一致すること
        assert_eq!(headers[2].1, "QDKvjWEDUSOQbljgZxQMxQ==");
        // 計算結果が呼び出し側の Option にも保存されること
        assert_eq!(
            computed_key_md5.as_deref(),
            Some("QDKvjWEDUSOQbljgZxQMxQ==")
        );
    }

    /// `add_sse_c_headers()` は algorithm / key が未指定の場合は何も追加しない
    #[test]
    fn test_add_sse_c_headers_none() {
        let mut headers = Vec::new();
        let mut computed_key_md5 = None;

        add_sse_c_headers(&mut headers, None, None, &mut computed_key_md5, false)
            .expect("有効なヘッダーであること");

        assert!(headers.is_empty());
        assert!(computed_key_md5.is_none());
    }

    /// `add_sse_c_headers()` は algorithm のみ指定時に algorithm ヘッダーのみ追加する
    #[test]
    fn test_add_sse_c_headers_algorithm_only() {
        let mut headers = Vec::new();
        let mut computed_key_md5 = None;

        add_sse_c_headers(
            &mut headers,
            Some("AES256"),
            None,
            &mut computed_key_md5,
            false,
        )
        .expect("有効なヘッダーであること");

        assert_eq!(headers.len(), 1);
        assert_eq!(
            headers[0],
            ("x-amz-server-side-encryption-customer-algorithm", "AES256")
        );
        assert!(computed_key_md5.is_none());
    }

    /// `add_sse_c_headers()` は key のみ指定時に key と MD5 ヘッダーを追加する
    #[test]
    fn test_add_sse_c_headers_key_only() {
        let mut headers = Vec::new();
        let mut computed_key_md5 = None;

        add_sse_c_headers(
            &mut headers,
            None,
            Some("MDEyMzQ1Njc4OWFiY2RlZg=="),
            &mut computed_key_md5,
            false,
        )
        .expect("有効なヘッダーであること");

        assert_eq!(headers.len(), 2);
        assert_eq!(
            headers[0],
            (
                "x-amz-server-side-encryption-customer-key",
                "MDEyMzQ1Njc4OWFiY2RlZg=="
            )
        );
        assert_eq!(
            headers[1].0,
            "x-amz-server-side-encryption-customer-key-md5"
        );
        assert_eq!(headers[1].1, "QDKvjWEDUSOQbljgZxQMxQ==");
    }

    /// `add_sse_c_headers()` は copy_source 指定時の不正な Base64 キーを拒否する
    ///
    /// エラー時はコピー元用の key ヘッダーが push 済みの状態になること、
    /// MD5 が未計算のままであることを検証する。
    #[test]
    fn test_add_sse_c_headers_copy_source_invalid_base64() {
        let mut headers = Vec::new();
        let mut computed_key_md5 = None;

        let result = add_sse_c_headers(
            &mut headers,
            None,
            Some("invalid!"),
            &mut computed_key_md5,
            true,
        );
        assert!(matches!(result, Err(Error::InvalidInput(_))));
        // コピー元用の key ヘッダーが push 済みの状態でエラーになること
        assert_eq!(headers.len(), 1);
        assert_eq!(
            headers[0].0,
            "x-amz-copy-source-server-side-encryption-customer-key"
        );
        assert!(computed_key_md5.is_none());
    }

    /// `add_sse_c_headers()` は不正な Base64 キーを Error::InvalidInput として拒否する
    #[test]
    fn test_add_sse_c_headers_invalid_base64() {
        let mut headers = Vec::new();
        let mut computed_key_md5 = None;

        let result = add_sse_c_headers(
            &mut headers,
            None,
            Some("invalid!"),
            &mut computed_key_md5,
            false,
        );
        assert!(matches!(result, Err(Error::InvalidInput(_))));
        // key ヘッダーは push 済みの状態でエラーになること、MD5 は未計算のままであること
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "x-amz-server-side-encryption-customer-key");
        assert!(computed_key_md5.is_none());
    }
}
