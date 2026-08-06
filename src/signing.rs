use crate::credential::Credentials;
pub(crate) use crate::datetime::UtcDateTime;

// -------------------------------------------------------
// 暗号プリミティブ (feature で切り替え)
// -------------------------------------------------------

/// SHA-256 ハッシュを計算する
///
/// 署名計算とチェックサム計算の両方で使用する共通関数。
/// 両方の feature が有効な場合は rust-crypto を優先する。
#[cfg(feature = "rust-crypto")]
pub(crate) fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    sha2::Sha256::digest(data).into()
}

#[cfg(all(feature = "aws_lc_rs", not(feature = "rust-crypto")))]
pub(crate) fn sha256(data: &[u8]) -> [u8; 32] {
    let d = aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA256, data);
    d.as_ref().try_into().expect("SHA-256 digest is 32 bytes")
}

/// HMAC-SHA256 を計算する
///
/// 両方の feature が有効な場合は rust-crypto を優先する
#[cfg(feature = "rust-crypto")]
fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    use hmac::{KeyInit, Mac};
    let mut mac =
        hmac::Hmac::<sha2::Sha256>::new_from_slice(key).expect("HMAC key length is invalid");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

#[cfg(all(feature = "aws_lc_rs", not(feature = "rust-crypto")))]
fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let key = aws_lc_rs::hmac::Key::new(aws_lc_rs::hmac::HMAC_SHA256, key);
    let tag = aws_lc_rs::hmac::sign(&key, data);
    tag.as_ref()
        .try_into()
        .expect("HMAC-SHA256 tag is 32 bytes")
}

// -------------------------------------------------------
// 署名ユーティリティ
// -------------------------------------------------------

/// SHA-256 ハッシュの 16 進数文字列を返す
pub(crate) fn hex_sha256(data: &[u8]) -> String {
    hex_encode(&sha256(data))
}

/// バイト列を小文字の 16 進数文字列に変換する
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        s.push(HEX_CHARS[(byte >> 4) as usize] as char);
        s.push(HEX_CHARS[(byte & 0x0f) as usize] as char);
    }
    s
}

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

/// URI パスをエンコードする (RFC 3986)
///
/// スラッシュはエンコードしない
pub(crate) fn uri_encode_path(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                result.push(byte as char);
            }
            _ => {
                result.push('%');
                result.push(HEX_UPPER[(byte >> 4) as usize] as char);
                result.push(HEX_UPPER[(byte & 0x0f) as usize] as char);
            }
        }
    }
    result
}

/// URI コンポーネントをエンコードする (RFC 3986)
///
/// スラッシュもエンコードする
pub(crate) fn uri_encode_component(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push('%');
                result.push(HEX_UPPER[(byte >> 4) as usize] as char);
                result.push(HEX_UPPER[(byte & 0x0f) as usize] as char);
            }
        }
    }
    result
}

const HEX_UPPER: &[u8; 16] = b"0123456789ABCDEF";

/// ヘッダー値の前後の空白を除去し、連続する空白を単一のスペースに畳む
fn fold_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.trim().chars() {
        if ch.is_ascii_whitespace() {
            if !prev_space {
                result.push(' ');
                prev_space = true;
            }
        } else {
            result.push(ch);
            prev_space = false;
        }
    }
    result
}

/// 署名対象ヘッダー値を正規化する
///
/// SigV4 仕様に従い前後の空白を除去して連続空白を単一スペースに畳み込む。
fn normalize_header_value(s: &str) -> String {
    fold_whitespace(s)
}

/// 正規クエリ文字列を構築する
///
/// パラメータをキーでソートし、各キーと値を URI エンコードして連結する
pub(crate) fn build_canonical_query_string(params: &[(&str, &str)]) -> String {
    if params.is_empty() {
        return String::new();
    }

    let mut sorted: Vec<_> = params
        .iter()
        .map(|(k, v)| (uri_encode_component(k), uri_encode_component(v)))
        .collect();
    sorted.sort();

    sorted
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&")
}

/// SigV4 の署名キーを導出する
///
/// 4 段階の HMAC-SHA256 で DateKey / DateRegionKey / DateRegionServiceKey / SigningKey を
/// 順に導出する。S3 専用ライブラリのため service は `s3` に固定する。
///
/// 仕様: https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-authenticating-requests.html
/// > Signing key derivation: DateKey = HMAC-SHA256("AWS4" + SecretAccessKey, Date);
/// > DateRegionKey = HMAC-SHA256(DateKey, Region);
/// > DateRegionServiceKey = HMAC-SHA256(DateRegionKey, Service);
/// > SigningKey = HMAC-SHA256(DateRegionServiceKey, "aws4_request")
///
/// 署名キー導出の手順は AWS の仕様変更により将来変わる可能性がある。
fn derive_signing_key(secret_key: &str, date_stamp: &str, region: &str) -> [u8; 32] {
    let date_key = hmac_sha256(
        format!("AWS4{secret_key}").as_bytes(),
        date_stamp.as_bytes(),
    );
    let date_region_key = hmac_sha256(&date_key, region.as_bytes());
    let date_region_service_key = hmac_sha256(&date_region_key, b"s3");
    hmac_sha256(&date_region_service_key, b"aws4_request")
}

/// 正規ヘッダーと署名対象ヘッダー名を構築する
///
/// headers は (小文字名, 値) のスライスで、ソート済みであること。
/// 戻り値は `(正規ヘッダー文字列, 署名対象ヘッダー名のセミコロン区切り文字列)` の順序。
/// SigV4 仕様に従いヘッダー値の前後の空白を除去し、連続する空白を単一のスペースに畳む。
fn build_canonical_headers(headers: &[(&str, &str)]) -> (String, String) {
    let canonical_headers_str: String = headers
        .iter()
        .map(|(name, value)| format!("{name}:{}\n", normalize_header_value(value)))
        .collect();

    let signed_headers: String = headers
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(";");

    (canonical_headers_str, signed_headers)
}

/// SigV4 のクレデンシャルスコープ文字列を構築する
///
/// S3 専用ライブラリのため service は `s3` に固定する。
/// 署名キー導出の `aws4_request` 終端と合わせ、Authorization ヘッダーの Credential と
/// Presigned URL の X-Amz-Credential の両方で利用する。
pub(crate) fn build_scope(date_stamp: &str, region: &str) -> String {
    format!("{date_stamp}/{region}/s3/aws4_request")
}

/// 署名の入力パラメータ
pub(crate) struct SigningParams<'a> {
    pub credentials: &'a Credentials,
    pub method: &'a str,
    pub canonical_uri: &'a str,
    pub canonical_query_string: &'a str,
    pub headers: &'a [(&'a str, &'a str)],
    pub payload_hash: &'a str,
    pub datetime: &'a UtcDateTime,
    pub region: &'a str,
}

/// AWS Signature V4 の Authorization ヘッダー値を計算する
///
/// headers は (小文字名, 値) のスライスで、ソート済みであること
pub(crate) fn compute_authorization(params: &SigningParams<'_>) -> String {
    let credentials = params.credentials;
    let method = params.method;
    let canonical_uri = params.canonical_uri;
    let canonical_query_string = params.canonical_query_string;
    let headers = params.headers;
    let payload_hash = params.payload_hash;
    let region = params.region;
    let date_stamp = params.datetime.date_stamp();
    let amz_date = params.datetime.iso8601();

    // 正規ヘッダーを構築する
    let (canonical_headers_str, signed_headers) = build_canonical_headers(headers);

    // 正規リクエストを構築する
    let canonical_request = format!(
        "{method}\n{canonical_uri}\n{canonical_query_string}\n{canonical_headers_str}\n{signed_headers}\n{payload_hash}"
    );

    // スコープを構築する
    let scope = build_scope(&date_stamp, region);

    // 署名対象文字列を構築する
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
        hex_sha256(canonical_request.as_bytes())
    );

    // 署名キーを導出する
    let signing_key = derive_signing_key(&credentials.secret_access_key, &date_stamp, region);

    // 署名を計算する
    let signature = hex_encode(&hmac_sha256(&signing_key, string_to_sign.as_bytes()));

    // Authorization ヘッダーを構成する
    format!(
        "AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={signature}",
        credentials.access_key_id
    )
}

/// Presigned URL 用の署名パラメータ
pub(crate) struct PresignParams<'a> {
    pub credentials: &'a Credentials,
    pub method: &'a str,
    pub canonical_uri: &'a str,
    /// 署名対象に含める全クエリパラメータ (X-Amz-* 含む、X-Amz-Signature 除く)
    pub query_params: &'a [(&'a str, &'a str)],
    pub headers: &'a [(&'a str, &'a str)],
    pub datetime: &'a UtcDateTime,
    pub region: &'a str,
}

/// Presigned URL 用の署名を計算する
///
/// headers は (小文字名, 値) のスライスで、ソート済みであること。
/// query_params には X-Amz-Algorithm 等の署名パラメータも含めた状態で渡す
pub(crate) fn compute_presigned_signature(params: &PresignParams<'_>) -> String {
    let date_stamp = params.datetime.date_stamp();
    let scope = build_scope(&date_stamp, params.region);

    let canonical_query_string = build_canonical_query_string(params.query_params);

    let (canonical_headers_str, signed_headers) = build_canonical_headers(params.headers);

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\nUNSIGNED-PAYLOAD",
        params.method,
        params.canonical_uri,
        canonical_query_string,
        canonical_headers_str,
        signed_headers
    );

    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{scope}\n{}",
        params.datetime.iso8601(),
        hex_sha256(canonical_request.as_bytes())
    );

    let signing_key = derive_signing_key(
        &params.credentials.secret_access_key,
        &date_stamp,
        params.region,
    );

    hex_encode(&hmac_sha256(&signing_key, string_to_sign.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // AWS 公式テストベクトルで使用するクレデンシャル (2013-05-24T00:00:00Z)
    const TEST_ACCESS_KEY_ID: &str = "AKIAIOSFODNN7EXAMPLE";
    const TEST_SECRET_ACCESS_KEY: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
    const TEST_TIMESTAMP: u64 = 1369353600;

    /// テストベクトル用のクレデンシャルを生成する
    fn test_credentials() -> Credentials {
        Credentials::new(
            TEST_ACCESS_KEY_ID,
            TEST_SECRET_ACCESS_KEY,
            None,
            None,
            "static",
        )
    }

    /// テストベクトル用の日時 (2013-05-24T00:00:00Z) を生成する
    fn test_datetime() -> UtcDateTime {
        UtcDateTime::from_unix_timestamp(TEST_TIMESTAMP).expect("valid timestamp")
    }

    #[test]
    fn test_hex_sha256_empty() {
        let hash = hex_sha256(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_uri_encode_path_simple() {
        assert_eq!(uri_encode_path("/foo/bar"), "/foo/bar");
        assert_eq!(uri_encode_path("/foo bar/baz"), "/foo%20bar/baz");
    }

    #[test]
    fn test_uri_encode_component() {
        assert_eq!(uri_encode_component("foo/bar"), "foo%2Fbar");
        assert_eq!(uri_encode_component("hello world"), "hello%20world");
    }

    #[test]
    fn test_build_canonical_query_string() {
        let params = vec![("prefix", "photos/"), ("max-keys", "10")];
        let result = build_canonical_query_string(&params);
        assert_eq!(result, "max-keys=10&prefix=photos%2F");
    }

    /// ヘッダー値の前後空白除去と連続空白の畳み込みを検証する
    ///
    /// SigV4 のヘッダー正規化の中核であり、公式テストベクトルでは空白を含む値が
    /// 使われないため単体で検証する
    #[test]
    fn test_build_canonical_headers_whitespace_folding() {
        let headers = vec![
            ("host", "examplebucket.s3.amazonaws.com"),
            ("x-amz-meta-foo", "  a   b  "),
        ];

        let (canonical_headers_str, signed_headers) = build_canonical_headers(&headers);

        // 前後の空白を除去し、連続する空白を単一のスペースに畳む
        assert_eq!(
            canonical_headers_str,
            "host:examplebucket.s3.amazonaws.com\nx-amz-meta-foo:a b\n"
        );
        assert_eq!(signed_headers, "host;x-amz-meta-foo");
    }

    /// AWS 公式テストベクトルに基づく署名検証
    ///
    /// 仕様: https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-authenticating-requests.html
    /// の GET Object 例に基づく
    #[test]
    fn test_compute_authorization() {
        let credentials = test_credentials();

        let datetime = test_datetime();

        // 空ボディの SHA-256 ハッシュ (AWS 公式テストベクトルの固定値)
        let payload_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

        let headers = vec![
            ("host", "examplebucket.s3.amazonaws.com"),
            ("range", "bytes=0-9"),
            ("x-amz-content-sha256", payload_hash),
            ("x-amz-date", "20130524T000000Z"),
        ];

        let auth = compute_authorization(&SigningParams {
            credentials: &credentials,
            method: "GET",
            canonical_uri: "/test.txt",
            canonical_query_string: "",
            headers: &headers,
            payload_hash,
            datetime: &datetime,
            region: "us-east-1",
        });

        assert!(auth.starts_with(
            "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20130524/us-east-1/s3/aws4_request"
        ));
        assert!(auth.contains("SignedHeaders=host;range;x-amz-content-sha256;x-amz-date"));
        // Signature は AWS 公式テストベクトルの値と完全一致すること
        assert!(auth.ends_with(
            ", Signature=f0e8bdb87c964420e857bd35b5d6ed310bd44f0170aba48dd91039c6036bdb41"
        ));
    }

    /// AWS 公式テストベクトルに基づく Presigned URL 署名検証
    ///
    /// 仕様: https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html
    /// の GET Object 例に基づく。署名計算が仕様から逸脱しないことを固定する回帰テスト
    #[test]
    fn test_compute_presigned_signature() {
        let credentials = test_credentials();

        let datetime = test_datetime();

        let headers = vec![("host", "examplebucket.s3.amazonaws.com")];

        let query_params = vec![
            ("X-Amz-Algorithm", "AWS4-HMAC-SHA256"),
            (
                "X-Amz-Credential",
                "AKIAIOSFODNN7EXAMPLE/20130524/us-east-1/s3/aws4_request",
            ),
            ("X-Amz-Date", "20130524T000000Z"),
            ("X-Amz-Expires", "86400"),
            ("X-Amz-SignedHeaders", "host"),
        ];

        let signature = compute_presigned_signature(&PresignParams {
            credentials: &credentials,
            method: "GET",
            canonical_uri: "/test.txt",
            query_params: &query_params,
            headers: &headers,
            datetime: &datetime,
            region: "us-east-1",
        });

        // AWS 公式テストベクトルの署名と完全一致することを検証する
        assert_eq!(
            signature,
            "aeeed9bbccd4d02ee5c0109b86d86835f995330da4c265957d157751f604d404"
        );
    }
}
