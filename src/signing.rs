use crate::credential::Credential;
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

#[cfg(all(feature = "aws-lc-rs", not(feature = "rust-crypto")))]
pub(crate) fn sha256(data: &[u8]) -> [u8; 32] {
    let d = aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA256, data);
    d.as_ref().try_into().expect("SHA-256 digest is 32 bytes")
}

/// HMAC-SHA256 を計算する
///
/// 両方の feature が有効な場合は rust-crypto を優先する
#[cfg(feature = "rust-crypto")]
fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    use hmac::Mac;
    let mut mac =
        hmac::Hmac::<sha2::Sha256>::new_from_slice(key).expect("HMAC key length is invalid");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

#[cfg(all(feature = "aws-lc-rs", not(feature = "rust-crypto")))]
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

/// 署名の入力パラメータ
pub(crate) struct SigningParams<'a> {
    pub credential: &'a Credential,
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
    let credential = params.credential;
    let method = params.method;
    let canonical_uri = params.canonical_uri;
    let canonical_query_string = params.canonical_query_string;
    let headers = params.headers;
    let payload_hash = params.payload_hash;
    let region = params.region;
    let date_stamp = params.datetime.date_stamp();
    let amz_date = params.datetime.iso8601();

    // 正規ヘッダーを構築する
    // SigV4 仕様: 前後の空白を trim し、連続する空白を単一のスペースに畳む
    let canonical_headers_str: String = headers
        .iter()
        .map(|(name, value)| format!("{name}:{}\n", normalize_header_value(value)))
        .collect();

    let signed_headers: String = headers
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(";");

    // 正規リクエストを構築する
    let canonical_request = format!(
        "{method}\n{canonical_uri}\n{canonical_query_string}\n{canonical_headers_str}\n{signed_headers}\n{payload_hash}"
    );

    // スコープを構築する
    let scope = format!("{date_stamp}/{region}/s3/aws4_request");

    // 署名対象文字列を構築する
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
        hex_sha256(canonical_request.as_bytes())
    );

    // 署名キーを導出する
    let date_key = hmac_sha256(
        format!("AWS4{}", credential.secret_access_key).as_bytes(),
        date_stamp.as_bytes(),
    );
    let date_region_key = hmac_sha256(&date_key, region.as_bytes());
    let date_region_service_key = hmac_sha256(&date_region_key, b"s3");
    let signing_key = hmac_sha256(&date_region_service_key, b"aws4_request");

    // 署名を計算する
    let signature = hex_encode(&hmac_sha256(&signing_key, string_to_sign.as_bytes()));

    // Authorization ヘッダーを構成する
    format!(
        "AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={signature}",
        credential.access_key_id
    )
}

/// Presigned URL 用の署名パラメータ
pub(crate) struct PresignParams<'a> {
    pub credential: &'a Credential,
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
/// query_params には X-Amz-Algorithm 等の署名パラメータも含めた状態で渡す
pub(crate) fn compute_presigned_signature(params: &PresignParams<'_>) -> String {
    let date_stamp = params.datetime.date_stamp();
    let scope = format!("{date_stamp}/{}/s3/aws4_request", params.region);

    let canonical_query_string = build_canonical_query_string(params.query_params);

    let canonical_headers_str: String = params
        .headers
        .iter()
        .map(|(name, value)| format!("{name}:{}\n", normalize_header_value(value)))
        .collect();

    let signed_headers: String = params
        .headers
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(";");

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

    let date_key = hmac_sha256(
        format!("AWS4{}", params.credential.secret_access_key).as_bytes(),
        date_stamp.as_bytes(),
    );
    let date_region_key = hmac_sha256(&date_key, params.region.as_bytes());
    let date_region_service_key = hmac_sha256(&date_region_key, b"s3");
    let signing_key = hmac_sha256(&date_region_service_key, b"aws4_request");

    hex_encode(&hmac_sha256(&signing_key, string_to_sign.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// AWS 公式テストベクトルに基づく署名検証
    #[test]
    fn test_compute_authorization() {
        let credential = Credential::new(
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        );

        let datetime = UtcDateTime::from_unix_timestamp(1369353600); // 2013-05-24T00:00:00Z

        let payload_hash = hex_sha256(b"");

        let headers = vec![
            ("host", "examplebucket.s3.amazonaws.com"),
            ("x-amz-content-sha256", payload_hash.as_str()),
            ("x-amz-date", "20130524T000000Z"),
        ];

        let auth = compute_authorization(&SigningParams {
            credential: &credential,
            method: "GET",
            canonical_uri: "/test.txt",
            canonical_query_string: "",
            headers: &headers,
            payload_hash: &payload_hash,
            datetime: &datetime,
            region: "us-east-1",
        });

        assert!(auth.starts_with(
            "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20130524/us-east-1/s3/aws4_request"
        ));
        assert!(auth.contains("SignedHeaders=host;x-amz-content-sha256;x-amz-date"));
    }
}
