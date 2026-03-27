mod abort_multipart_upload;
mod complete_multipart_upload;
mod copy_object;
mod create_bucket;
mod create_multipart_upload;
mod delete_bucket;
mod delete_bucket_lifecycle_configuration;
mod delete_bucket_policy;
mod delete_bucket_tagging;
mod delete_object;
mod delete_objects;
mod delete_public_access_block;
mod get_bucket_lifecycle_configuration;
mod get_bucket_policy;
mod get_bucket_tagging;
mod get_bucket_versioning;
mod get_object;
mod get_public_access_block;
mod head_bucket;
mod head_object;
mod list_buckets;
mod list_multipart_uploads;
mod list_objects_v2;
mod list_parts;
mod put_bucket_lifecycle_configuration;
mod put_bucket_policy;
mod put_bucket_tagging;
mod put_bucket_versioning;
mod put_object;
mod put_public_access_block;
mod upload_part;

pub use abort_multipart_upload::AbortMultipartUploadFluentBuilder;
pub use complete_multipart_upload::CompleteMultipartUploadFluentBuilder;
pub use copy_object::CopyObjectFluentBuilder;
pub use create_bucket::CreateBucketFluentBuilder;
pub use create_multipart_upload::CreateMultipartUploadFluentBuilder;
pub use delete_bucket::DeleteBucketFluentBuilder;
pub use delete_bucket_lifecycle_configuration::DeleteBucketLifecycleConfigurationFluentBuilder;
pub use delete_bucket_policy::DeleteBucketPolicyFluentBuilder;
pub use delete_bucket_tagging::DeleteBucketTaggingFluentBuilder;
pub use delete_object::DeleteObjectFluentBuilder;
pub use delete_objects::DeleteObjectsFluentBuilder;
pub use delete_public_access_block::DeletePublicAccessBlockFluentBuilder;
pub use get_bucket_lifecycle_configuration::GetBucketLifecycleConfigurationFluentBuilder;
pub use get_bucket_policy::GetBucketPolicyFluentBuilder;
pub use get_bucket_tagging::GetBucketTaggingFluentBuilder;
pub use get_bucket_versioning::GetBucketVersioningFluentBuilder;
pub use get_object::GetObjectFluentBuilder;
pub use get_public_access_block::GetPublicAccessBlockFluentBuilder;
pub use head_bucket::HeadBucketFluentBuilder;
pub use head_object::HeadObjectFluentBuilder;
pub use list_buckets::ListBucketsFluentBuilder;
pub use list_multipart_uploads::ListMultipartUploadsFluentBuilder;
pub use list_objects_v2::ListObjectsV2FluentBuilder;
pub use list_parts::ListPartsFluentBuilder;
pub use put_bucket_lifecycle_configuration::PutBucketLifecycleConfigurationFluentBuilder;
pub use put_bucket_policy::PutBucketPolicyFluentBuilder;
pub use put_bucket_tagging::PutBucketTaggingFluentBuilder;
pub use put_bucket_versioning::PutBucketVersioningFluentBuilder;
pub use put_object::PutObjectFluentBuilder;
pub use put_public_access_block::PutPublicAccessBlockFluentBuilder;
pub use upload_part::UploadPartFluentBuilder;

use crate::client::S3Client;
use crate::credential::Credential;
use crate::error::Error;
use crate::signing::{
    PresignParams, SigningParams, UtcDateTime, build_canonical_query_string, compute_authorization,
    compute_presigned_signature, hex_sha256, uri_encode_path,
};

// -------------------------------------------------------
// PresignedRequest
// -------------------------------------------------------

/// Presigned リクエスト
///
/// URL だけでなく、リクエストに必要なボディも保持する。
/// GET / HEAD / DELETE など body が不要な場合は `body` は空。
/// CompleteMultipartUpload のように POST body が必要な場合は XML 等が入る。
#[derive(Debug, Clone)]
pub struct PresignedRequest {
    /// Presigned URL
    pub url: String,
    /// HTTP メソッド
    pub method: String,
    /// リクエスト時に付与が必要な header (署名対象に含まれる)
    pub headers: Vec<(String, String)>,
    /// リクエストボディ (不要な場合は空)
    pub body: Vec<u8>,
}

// -------------------------------------------------------
// S3Request / S3Response
// -------------------------------------------------------

/// 署名済み S3 リクエスト
///
/// `build_request()` で構築する。
/// 利用者は各フィールドを使って任意の HTTP クライアントでリクエストを送信する。
#[derive(Debug, Clone)]
pub struct S3Request {
    /// HTTP メソッド (GET, PUT, DELETE, POST, HEAD)
    pub method: String,
    /// リクエスト URI (パス + クエリ文字列)
    pub uri: String,
    /// HTTP リクエストヘッダー (名前, 値) のリスト (署名済み)
    pub headers: Vec<(String, String)>,
    /// リクエストボディ
    pub body: Vec<u8>,
    /// 接続先ホスト名
    pub host: String,
    /// 接続先ポート番号
    pub port: u16,
    /// HTTPS を使用するかどうか
    pub https: bool,
    /// TLS 証明書の検証を無視する (テスト環境向け)
    pub ignore_cert_check: bool,
    /// レスポンスにボディがないことを期待するか (HEAD リクエスト)
    pub expect_no_body: bool,
}

/// S3 レスポンス
///
/// HTTP レスポンスのステータスコード、ヘッダー、ボディを保持する。
/// 利用者が任意の HTTP クライアントから構築して `parse_response()` に渡す。
#[derive(Debug, Clone)]
pub struct S3Response {
    /// HTTP ステータスコード
    pub status_code: u16,
    /// HTTP レスポンスヘッダー (名前, 値) のリスト
    pub headers: Vec<(String, String)>,
    /// レスポンスボディ
    pub body: Vec<u8>,
}

impl S3Response {
    /// レスポンスが成功 (2xx) かどうかを返す
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }

    /// 指定した名前のヘッダー値を返す (大文字小文字を区別しない)
    pub fn get_header(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == name_lower)
            .map(|(_, v)| v.as_str())
    }

    /// Content-Length ヘッダーの値を返す
    pub fn content_length(&self) -> Option<u64> {
        self.get_header("content-length")
            .and_then(|v| v.parse().ok())
    }

    /// x-amz-meta-* ヘッダーからカスタムメタデータを抽出する
    ///
    /// メタデータが存在しない場合は None を返す。
    pub fn extract_metadata(&self) -> Option<std::collections::HashMap<String, String>> {
        let prefix = "x-amz-meta-";
        let map: std::collections::HashMap<String, String> = self
            .headers
            .iter()
            .filter_map(|(k, v)| {
                let lower = k.to_ascii_lowercase();
                lower
                    .strip_prefix(prefix)
                    .map(|key| (key.to_string(), v.clone()))
            })
            .collect();
        if map.is_empty() { None } else { Some(map) }
    }
}

// -------------------------------------------------------
// 設定参照 (ライフタイム付き、Sans I/O で使用)
// -------------------------------------------------------

/// S3Client の設定への参照
pub(crate) struct S3ClientConfig<'a> {
    pub(crate) region: &'a str,
    pub(crate) credential: &'a Credential,
    /// スキームを除去したホスト名 (ポート含む場合あり)
    pub(crate) endpoint: Option<&'a str>,
    pub(crate) use_path_style: bool,
    /// HTTPS を使用するかどうか (endpoint のスキームから判定)
    pub(crate) https: bool,
    /// TLS 証明書の検証を無視する
    pub(crate) ignore_cert_check: bool,
}

/// endpoint 文字列からスキームを解析する
///
/// - `"http://localhost:9000"` → `(false, "localhost:9000")`
/// - `"https://minio.example.com"` → `(true, "minio.example.com")`
/// - `"minio.example.com"` → `(true, "minio.example.com")` (スキーム省略時は HTTPS)
fn parse_endpoint_scheme(endpoint: &str) -> (bool, &str) {
    if let Some(rest) = endpoint.strip_prefix("http://") {
        (false, rest)
    } else if let Some(rest) = endpoint.strip_prefix("https://") {
        (true, rest)
    } else {
        (true, endpoint)
    }
}

impl S3Client {
    pub(crate) fn config_ref(&self) -> S3ClientConfig<'_> {
        let (https, endpoint) = match self.config.endpoint.as_deref() {
            Some(ep) => {
                let (https, host) = parse_endpoint_scheme(ep);
                (https, Some(host))
            }
            None => (true, None),
        };

        S3ClientConfig {
            region: &self.config.region,
            credential: &self.config.credential,
            endpoint,
            use_path_style: self.config.use_path_style,
            https,
            ignore_cert_check: self.config.ignore_cert_check,
        }
    }
}

// -------------------------------------------------------
// 署名済みリクエスト構築 (Sans I/O)
// -------------------------------------------------------

/// 署名済みバケットリクエストを構築する
pub(crate) fn build_signed_request(
    config: &S3ClientConfig<'_>,
    method: &str,
    bucket: &str,
    key: &str,
    extra_headers: &[(&str, &str)],
    body: &[u8],
    query_params: Option<&[(&str, &str)]>,
) -> S3Request {
    let host = host_for_bucket(config, bucket);
    let path = path_for_key(config, bucket, key);
    build_signed_request_inner(
        config,
        method,
        &host,
        path,
        extra_headers,
        body,
        query_params,
    )
}

/// 署名済みサービスレベルリクエストを構築する (バケットなし)
pub(crate) fn build_signed_service_request(
    config: &S3ClientConfig<'_>,
    method: &str,
    path: &str,
    extra_headers: &[(&str, &str)],
    body: &[u8],
    query_params: Option<&[(&str, &str)]>,
) -> S3Request {
    let host = service_host(config);
    build_signed_request_inner(
        config,
        method,
        &host,
        path.to_string(),
        extra_headers,
        body,
        query_params,
    )
}

/// 署名済みリクエスト構築の共通処理
fn build_signed_request_inner(
    config: &S3ClientConfig<'_>,
    method: &str,
    host: &str,
    path: String,
    extra_headers: &[(&str, &str)],
    body: &[u8],
    query_params: Option<&[(&str, &str)]>,
) -> S3Request {
    let connect_host = extract_connect_host(host);
    let port = extract_port(config);
    let datetime = UtcDateTime::now();
    let amz_date = datetime.iso8601();
    let payload_hash = hex_sha256(body);

    let canonical_query_string = query_params
        .map(build_canonical_query_string)
        .unwrap_or_default();

    let mut sign_headers: Vec<(&str, String)> = Vec::new();
    for &(name, value) in extra_headers {
        sign_headers.push((name, value.to_string()));
    }
    sign_headers.push(("host", host.to_string()));
    sign_headers.push(("x-amz-content-sha256", payload_hash.clone()));
    sign_headers.push(("x-amz-date", amz_date.clone()));
    if let Some(token) = config.credential.session_token() {
        sign_headers.push(("x-amz-security-token", token.to_string()));
    }
    sign_headers.sort_by(|a, b| a.0.cmp(b.0));

    let headers_for_signing: Vec<(&str, &str)> = sign_headers
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();

    let authorization = compute_authorization(&SigningParams {
        credential: config.credential,
        method,
        canonical_uri: &path,
        canonical_query_string: &canonical_query_string,
        headers: &headers_for_signing,
        payload_hash: &payload_hash,
        datetime: &datetime,
        region: config.region,
    });

    let uri = if canonical_query_string.is_empty() {
        path
    } else {
        format!("{path}?{canonical_query_string}")
    };

    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("Host".to_string(), host.to_string()));
    for &(name, value) in extra_headers {
        headers.push((name.to_string(), value.to_string()));
    }
    headers.push(("x-amz-content-sha256".to_string(), payload_hash));
    headers.push(("x-amz-date".to_string(), amz_date));
    if let Some(token) = config.credential.session_token() {
        headers.push(("x-amz-security-token".to_string(), token.to_string()));
    }
    headers.push(("Authorization".to_string(), authorization));

    S3Request {
        method: method.to_string(),
        uri,
        headers,
        body: body.to_vec(),
        host: connect_host,
        port,
        https: config.https,
        ignore_cert_check: config.ignore_cert_check,
        expect_no_body: method == "HEAD",
    }
}

/// Presigned URL を生成する
///
/// `extra_headers` は署名対象に含める追加 header (SSE-C 等)。
/// リクエスト時にも同じ header を付与する必要がある。
pub(crate) fn build_presigned_url(
    config: &S3ClientConfig<'_>,
    method: &str,
    bucket: &str,
    key: &str,
    expires_in_secs: u64,
    extra_query_params: &[(&str, &str)],
    extra_headers: &[(&str, &str)],
) -> String {
    let host = host_for_bucket(config, bucket);
    let path = path_for_key(config, bucket, key);
    let datetime = UtcDateTime::now();
    let date_stamp = datetime.date_stamp();
    let amz_date = datetime.iso8601();
    let scope = format!("{date_stamp}/{}/s3/aws4_request", config.region);
    let credential_value = format!("{}/{scope}", config.credential.access_key_id);
    let expires_str = expires_in_secs.to_string();

    // 署名対象 header を構築する (host は必須)
    let mut headers: Vec<(&str, &str)> = vec![("host", host.as_str())];
    for &(name, value) in extra_headers {
        headers.push((name, value));
    }
    // header 名でソートする (canonical headers の要件)
    headers.sort_by_key(|&(name, _)| name);

    // X-Amz-SignedHeaders を構築する
    let signed_headers_value: String = headers
        .iter()
        .map(|&(name, _)| name)
        .collect::<Vec<_>>()
        .join(";");

    let session_token = config.credential.session_token().map(String::from);
    let mut query_params: Vec<(&str, &str)> = vec![
        ("X-Amz-Algorithm", "AWS4-HMAC-SHA256"),
        ("X-Amz-Credential", &credential_value),
        ("X-Amz-Date", &amz_date),
        ("X-Amz-Expires", &expires_str),
        ("X-Amz-SignedHeaders", &signed_headers_value),
    ];
    if let Some(ref token) = session_token {
        query_params.push(("X-Amz-Security-Token", token));
    }
    query_params.extend_from_slice(extra_query_params);

    let signature = compute_presigned_signature(&PresignParams {
        credential: config.credential,
        method,
        canonical_uri: &path,
        query_params: &query_params,
        headers: &headers,
        datetime: &datetime,
        region: config.region,
    });

    let canonical_query_string = build_canonical_query_string(&query_params);
    let scheme = if config.https { "https" } else { "http" };
    format!("{scheme}://{host}{path}?{canonical_query_string}&X-Amz-Signature={signature}")
}

// -------------------------------------------------------
// ホスト・パス計算
// -------------------------------------------------------

fn service_host(config: &S3ClientConfig<'_>) -> String {
    config
        .endpoint
        .map(String::from)
        .unwrap_or_else(|| format!("s3.{}.amazonaws.com", config.region))
}

/// HTTPS かつバケット名にドットを含む場合は path-style にフォールバックが必要
fn use_path_style_for_bucket(config: &S3ClientConfig<'_>, bucket: &str) -> bool {
    config.use_path_style || (config.https && bucket.contains('.'))
}

fn host_for_bucket(config: &S3ClientConfig<'_>, bucket: &str) -> String {
    let base = service_host(config);
    if use_path_style_for_bucket(config, bucket) {
        base
    } else {
        format!("{bucket}.{base}")
    }
}

/// ホスト文字列からポート部分を除去して接続先ホスト名を返す
///
/// IPv6 アドレス (`[::1]:9000` や `[::1]`) を正しく処理する
fn extract_connect_host(host: &str) -> String {
    if let Some(end) = host.find(']') {
        // IPv6: `[::1]:9000` → `[::1]`, `[::1]` → `[::1]`
        host[..=end].to_string()
    } else if let Some(pos) = host.rfind(':') {
        // IPv4 / ホスト名でポート付き: `localhost:9000` → `localhost`
        // ただしコロンが含まれていてもポート部分が数値でなければホスト名全体を返す
        if host[pos + 1..].parse::<u16>().is_ok() {
            host[..pos].to_string()
        } else {
            host.to_string()
        }
    } else {
        host.to_string()
    }
}

/// endpoint からポートを抽出する (明示的なポートがない場合は HTTPS なら 443、HTTP なら 80)
///
/// IPv6 アドレス (`[::1]:9000`) を正しく処理する
fn extract_port(config: &S3ClientConfig<'_>) -> u16 {
    if let Some(endpoint) = config.endpoint
        && let Some(port) = parse_port_from_authority(endpoint)
    {
        return port;
    }
    if config.https { 443 } else { 80 }
}

/// authority 文字列からポートを解析する
///
/// `[::1]:9000` → `Some(9000)`, `localhost:9000` → `Some(9000)`, `[::1]` → `None`
fn parse_port_from_authority(authority: &str) -> Option<u16> {
    if authority.starts_with('[') {
        // IPv6: `]` の後に `:port` があればポート
        let after_bracket = authority.find(']')?;
        let rest = &authority[after_bracket + 1..];
        let port_str = rest.strip_prefix(':')?;
        port_str.parse().ok()
    } else {
        // IPv4 / ホスト名: 最後の `:` 以降がポート
        let port_str = authority.rsplit(':').next()?;
        port_str.parse().ok()
    }
}

fn path_for_key(config: &S3ClientConfig<'_>, bucket: &str, key: &str) -> String {
    let encoded_key = uri_encode_path(key);
    if use_path_style_for_bucket(config, bucket) {
        if encoded_key.is_empty() {
            format!("/{bucket}")
        } else {
            format!("/{bucket}/{encoded_key}")
        }
    } else if encoded_key.is_empty() {
        "/".to_string()
    } else {
        format!("/{encoded_key}")
    }
}

// -------------------------------------------------------
// ヘルパー関数
// -------------------------------------------------------

/// 必須パラメータのバリデーション
pub(crate) fn required<'a>(value: Option<&'a str>, name: &str) -> Result<&'a str, Error> {
    value.ok_or_else(|| Error::InvalidInput(format!("{name} is required")))
}

/// Presigned URL の最小有効期限 (秒)
///
/// https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html
const PRESIGN_MIN_EXPIRES_SECS: u64 = 1;
/// Presigned URL の最大有効期限 (秒) — 7 日間
///
/// https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html
const PRESIGN_MAX_EXPIRES_SECS: u64 = 604800;

/// Presigned URL の有効期限を検証する (1〜604800 秒)
pub(crate) fn validate_presign_expires(expires_in_secs: u64) -> Result<(), Error> {
    if !(PRESIGN_MIN_EXPIRES_SECS..=PRESIGN_MAX_EXPIRES_SECS).contains(&expires_in_secs) {
        return Err(Error::InvalidInput(format!(
            "expires_in_secs must be between {PRESIGN_MIN_EXPIRES_SECS} and {PRESIGN_MAX_EXPIRES_SECS} (7 days)"
        )));
    }
    Ok(())
}

/// 2xx レスポンスのボディに `<Error>` が含まれていないか検査する
///
/// CompleteMultipartUpload と CopyObject は 200 OK でボディにエラーを返すことがある。
/// `<Error>` ルートタグの存在を確認してから `<Code>` を抽出する。
pub(crate) fn check_body_error(response: &S3Response) -> Result<(), Error> {
    if let Ok(text) = std::str::from_utf8(&response.body)
        && crate::xml::has_error_root(text)
    {
        return Err(parse_error_response_with_status(
            response.status_code,
            &response.body,
        ));
    }
    Ok(())
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
        .unwrap_or_else(|| ("UnknownError".to_string(), "unknown error".to_string()));

    Error::S3 {
        status_code,
        code,
        message,
    }
}

fn parse_s3_error_xml(body: &[u8]) -> Option<(String, String)> {
    crate::xml::parse_s3_error(body)
}

pub(crate) fn base64_md5(data: &[u8]) -> String {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use md5::{Digest, Md5};
    let hash = Md5::digest(data);
    STANDARD.encode(hash.as_slice())
}

/// SSE-C キー (Base64) から MD5 (Base64) を自動計算する
///
/// sse_customer_key が指定されていて sse_customer_key_md5 が未指定の場合に
/// 自動的に MD5 を計算する。
pub(crate) fn compute_sse_c_key_md5(base64_key: &str) -> Result<String, Error> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use md5::{Digest, Md5};
    let key_bytes = STANDARD
        .decode(base64_key)
        .map_err(|_| Error::InvalidInput("SSE-C key must be valid Base64".to_string()))?;
    let hash = Md5::digest(&key_bytes);
    Ok(STANDARD.encode(hash.as_slice()))
}
