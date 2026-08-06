mod abort_multipart_upload;
mod complete_multipart_upload;
mod copy_object;
mod create_bucket;
mod create_multipart_upload;
mod delete_bucket;
mod delete_bucket_cors;
mod delete_bucket_encryption;
mod delete_bucket_lifecycle;
mod delete_bucket_ownership_controls;
mod delete_bucket_policy;
mod delete_bucket_tagging;
mod delete_bucket_website;
mod delete_object;
mod delete_object_tagging;
mod delete_objects;
mod delete_public_access_block;
mod endpoint;
mod get_bucket_cors;
mod get_bucket_encryption;
mod get_bucket_lifecycle_configuration;
mod get_bucket_notification_configuration;
mod get_bucket_ownership_controls;
mod get_bucket_policy;
mod get_bucket_tagging;
mod get_bucket_versioning;
mod get_bucket_website;
mod get_object;
mod get_object_legal_hold;
mod get_object_lock_configuration;
mod get_object_retention;
mod get_object_tagging;
mod get_public_access_block;
mod head_bucket;
mod head_object;
mod list_buckets;
mod list_multipart_uploads;
mod list_object_versions;
mod list_objects_v1;
mod list_objects_v2;
mod list_parts;
mod put_bucket_cors;
mod put_bucket_encryption;
mod put_bucket_lifecycle_configuration;
mod put_bucket_notification_configuration;
mod put_bucket_ownership_controls;
mod put_bucket_policy;
mod put_bucket_tagging;
mod put_bucket_versioning;
mod put_bucket_website;
mod put_object;
mod put_object_legal_hold;
mod put_object_lock_configuration;
mod put_object_retention;
mod put_object_tagging;
mod put_public_access_block;
mod upload_part;
mod upload_part_copy;
mod util;

pub use abort_multipart_upload::AbortMultipartUploadFluentBuilder;
pub use complete_multipart_upload::CompleteMultipartUploadFluentBuilder;
pub use copy_object::CopyObjectFluentBuilder;
pub use create_bucket::CreateBucketFluentBuilder;
pub use create_multipart_upload::CreateMultipartUploadFluentBuilder;
pub use delete_bucket::DeleteBucketFluentBuilder;
pub use delete_bucket_cors::DeleteBucketCorsFluentBuilder;
pub use delete_bucket_encryption::DeleteBucketEncryptionFluentBuilder;
pub use delete_bucket_lifecycle::DeleteBucketLifecycleFluentBuilder;
pub use delete_bucket_ownership_controls::DeleteBucketOwnershipControlsFluentBuilder;
pub use delete_bucket_policy::DeleteBucketPolicyFluentBuilder;
pub use delete_bucket_tagging::DeleteBucketTaggingFluentBuilder;
pub use delete_bucket_website::DeleteBucketWebsiteFluentBuilder;
pub use delete_object::DeleteObjectFluentBuilder;
pub use delete_object_tagging::DeleteObjectTaggingFluentBuilder;
pub use delete_objects::DeleteObjectsFluentBuilder;
pub use delete_public_access_block::DeletePublicAccessBlockFluentBuilder;
pub use get_bucket_cors::GetBucketCorsFluentBuilder;
pub use get_bucket_encryption::GetBucketEncryptionFluentBuilder;
pub use get_bucket_lifecycle_configuration::GetBucketLifecycleConfigurationFluentBuilder;
pub use get_bucket_notification_configuration::GetBucketNotificationConfigurationFluentBuilder;
pub use get_bucket_ownership_controls::GetBucketOwnershipControlsFluentBuilder;
pub use get_bucket_policy::GetBucketPolicyFluentBuilder;
pub use get_bucket_tagging::GetBucketTaggingFluentBuilder;
pub use get_bucket_versioning::GetBucketVersioningFluentBuilder;
pub use get_bucket_website::GetBucketWebsiteFluentBuilder;
pub use get_object::GetObjectFluentBuilder;
pub use get_object_legal_hold::GetObjectLegalHoldFluentBuilder;
pub use get_object_lock_configuration::GetObjectLockConfigurationFluentBuilder;
pub use get_object_retention::GetObjectRetentionFluentBuilder;
pub use get_object_tagging::GetObjectTaggingFluentBuilder;
pub use get_public_access_block::GetPublicAccessBlockFluentBuilder;
pub use head_bucket::HeadBucketFluentBuilder;
pub use head_object::HeadObjectFluentBuilder;
pub use list_buckets::ListBucketsFluentBuilder;
pub use list_multipart_uploads::ListMultipartUploadsFluentBuilder;
pub use list_object_versions::ListObjectVersionsFluentBuilder;
pub use list_objects_v1::ListObjectsFluentBuilder;
pub use list_objects_v2::ListObjectsV2FluentBuilder;
pub use list_parts::ListPartsFluentBuilder;
pub use put_bucket_cors::PutBucketCorsFluentBuilder;
pub use put_bucket_encryption::PutBucketEncryptionFluentBuilder;
pub use put_bucket_lifecycle_configuration::PutBucketLifecycleConfigurationFluentBuilder;
pub use put_bucket_notification_configuration::PutBucketNotificationConfigurationFluentBuilder;
pub use put_bucket_ownership_controls::PutBucketOwnershipControlsFluentBuilder;
pub use put_bucket_policy::PutBucketPolicyFluentBuilder;
pub use put_bucket_tagging::PutBucketTaggingFluentBuilder;
pub use put_bucket_versioning::PutBucketVersioningFluentBuilder;
pub use put_bucket_website::PutBucketWebsiteFluentBuilder;
pub use put_object::PutObjectFluentBuilder;
pub use put_object_legal_hold::PutObjectLegalHoldFluentBuilder;
pub use put_object_lock_configuration::PutObjectLockConfigurationFluentBuilder;
pub use put_object_retention::PutObjectRetentionFluentBuilder;
pub use put_object_tagging::PutObjectTaggingFluentBuilder;
pub use put_public_access_block::PutPublicAccessBlockFluentBuilder;
pub use upload_part::UploadPartFluentBuilder;
pub use upload_part_copy::UploadPartCopyFluentBuilder;

pub use crate::request::{PresignedRequest, S3Request, S3Response};
pub(crate) use util::{
    add_sse_c_headers, base64_md5, check_body_error, extract_xml_common_prefixes,
    extract_xml_objects, head_error_from_status, parse_error_response, required,
    validate_part_number, validate_presign_expires, xml_body_text,
};

use crate::client::Client;
use crate::error::Error;
use crate::signing::{
    PresignParams, SigningParams, UtcDateTime, build_canonical_query_string, build_scope,
    compute_authorization, compute_presigned_signature, hex_sha256,
};
use endpoint::{
    ClientConfig, extract_connect_host, extract_port, host_for_bucket, parse_endpoint_scheme,
    path_for_key, service_host,
};

// -------------------------------------------------------
// 設定参照の構築 (ライフタイム付き、Sans I/O で使用)
// -------------------------------------------------------

impl Client {
    pub(crate) fn config_ref(&self) -> ClientConfig<'_> {
        let (https, endpoint) = match self.config.endpoint.as_deref() {
            Some(ep) => {
                let (https, host) = parse_endpoint_scheme(ep);
                (https, Some(host))
            }
            None => (true, None),
        };

        ClientConfig {
            region: &self.config.region,
            credentials: &self.config.credentials_provider,
            endpoint,
            force_path_style: self.config.force_path_style,
            https,
            ignore_cert_check: self.config.ignore_cert_check,
        }
    }
}

// -------------------------------------------------------
// 署名済みリクエスト構築 (Sans I/O)
// -------------------------------------------------------

/// 署名済みバケットリクエストを構築する
///
/// `now` は `x-amz-date` ヘッダーおよびクレデンシャルスコープに使う現在時刻。
/// Sans I/O 原則のため呼び出し側で `SystemTime::now()` を取得して渡す。
#[expect(clippy::too_many_arguments)]
pub(crate) fn build_signed_request(
    config: &ClientConfig<'_>,
    method: &str,
    bucket: &str,
    key: &str,
    extra_headers: &[(&str, &str)],
    body: &[u8],
    query_params: Option<&[(&str, &str)]>,
    now: std::time::SystemTime,
) -> Result<S3Request, Error> {
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
        now,
    )
}

/// 署名済みサービスレベルリクエストを構築する (バケットなし)
pub(crate) fn build_signed_service_request(
    config: &ClientConfig<'_>,
    method: &str,
    path: &str,
    extra_headers: &[(&str, &str)],
    body: &[u8],
    query_params: Option<&[(&str, &str)]>,
    now: std::time::SystemTime,
) -> Result<S3Request, Error> {
    let host = service_host(config);
    build_signed_request_inner(
        config,
        method,
        &host,
        path.to_string(),
        extra_headers,
        body,
        query_params,
        now,
    )
}

/// 署名済みリクエスト構築の共通処理
#[expect(clippy::too_many_arguments)]
fn build_signed_request_inner(
    config: &ClientConfig<'_>,
    method: &str,
    host: &str,
    path: String,
    extra_headers: &[(&str, &str)],
    body: &[u8],
    query_params: Option<&[(&str, &str)]>,
    now: std::time::SystemTime,
) -> Result<S3Request, Error> {
    let connect_host = extract_connect_host(host);
    let port = extract_port(config);
    let datetime = UtcDateTime::from_system_time(now)?;
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
    if let Some(token) = config.credentials.session_token() {
        sign_headers.push(("x-amz-security-token", token.to_string()));
    }
    sign_headers.sort_by(|a, b| a.0.cmp(b.0));

    let headers_for_signing: Vec<(&str, &str)> = sign_headers
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();

    let authorization = compute_authorization(&SigningParams {
        credentials: config.credentials,
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
    if let Some(token) = config.credentials.session_token() {
        headers.push(("x-amz-security-token".to_string(), token.to_string()));
    }
    headers.push(("Authorization".to_string(), authorization));

    Ok(S3Request {
        method: method.to_string(),
        uri,
        headers,
        body: body.to_vec(),
        host: connect_host,
        port,
        https: config.https,
        ignore_cert_check: config.ignore_cert_check,
        expect_no_body: method == "HEAD",
    })
}

/// Presigned URL を生成する
///
/// `extra_headers` は署名対象に含める追加 header (SSE-C 等)。
/// リクエスト時にも同じ header を付与する必要がある。
/// `now` は `X-Amz-Date` クエリパラメータおよびクレデンシャルスコープに使う現在時刻。
#[expect(clippy::too_many_arguments)]
pub(crate) fn build_presigned_url(
    config: &ClientConfig<'_>,
    method: &str,
    bucket: &str,
    key: &str,
    expires_in_secs: u64,
    extra_query_params: &[(&str, &str)],
    extra_headers: &[(&str, &str)],
    now: std::time::SystemTime,
) -> Result<String, Error> {
    let host = host_for_bucket(config, bucket);
    let path = path_for_key(config, bucket, key);
    let datetime = UtcDateTime::from_system_time(now)?;
    let date_stamp = datetime.date_stamp();
    let amz_date = datetime.iso8601();
    let scope = build_scope(&date_stamp, config.region);
    let credential_value = format!("{}/{scope}", config.credentials.access_key_id);
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

    let session_token = config.credentials.session_token().map(String::from);
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
        credentials: config.credentials,
        method,
        canonical_uri: &path,
        query_params: &query_params,
        headers: &headers,
        datetime: &datetime,
        region: config.region,
    });

    let canonical_query_string = build_canonical_query_string(&query_params);
    let scheme = if config.https { "https" } else { "http" };
    Ok(format!(
        "{scheme}://{host}{path}?{canonical_query_string}&X-Amz-Signature={signature}"
    ))
}

#[cfg(test)]
mod sans_io_tests {
    use super::*;
    use crate::client::{Client, Config};
    use crate::credential::Credentials;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn fixed_now() -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1234567890) // 2009-02-13T23:31:30Z
    }

    fn build_test_client() -> Client {
        let config = Config::builder()
            .region("us-east-1")
            .credentials_provider(Credentials::new(
                "AKIAIOSFODNN7EXAMPLE",
                "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
                None,
                None,
                "test",
            ))
            .build()
            .expect("Config::build");
        Client::from_conf(config)
    }

    /// 同じ `now` を渡せば `build_request` は決定的な署名値を返す
    ///
    /// Sans I/O 原則の核心。同一入力に対して同一出力 (参照透過性) を満たす。
    #[test]
    fn test_build_request_is_deterministic_for_fixed_now() {
        let client = build_test_client();
        let now = fixed_now();

        let a = client
            .get_object()
            .bucket("examplebucket")
            .key("test.txt")
            .build_request(now)
            .expect("build_request");
        let b = client
            .get_object()
            .bucket("examplebucket")
            .key("test.txt")
            .build_request(now)
            .expect("build_request");

        // ヘッダーは順序付き Vec なので等値性が成り立つ
        assert_eq!(a.method, b.method);
        assert_eq!(a.uri, b.uri);
        assert_eq!(a.headers, b.headers);
        assert_eq!(a.body, b.body);
    }

    /// 異なる `now` を渡せば `x-amz-date` が変わり、署名値も変わる
    #[test]
    fn test_build_request_varies_with_now() {
        let client = build_test_client();
        let a = client
            .get_object()
            .bucket("examplebucket")
            .key("test.txt")
            .build_request(UNIX_EPOCH + Duration::from_secs(1234567890))
            .expect("build_request");
        let b = client
            .get_object()
            .bucket("examplebucket")
            .key("test.txt")
            .build_request(UNIX_EPOCH + Duration::from_secs(1234567891))
            .expect("build_request");

        // x-amz-date は秒精度で異なるためヘッダー全体も異なる
        assert_ne!(a.headers, b.headers);
    }

    /// 同じ `now` を渡せば `presigned` も決定的な URL を返す
    #[test]
    fn test_presigned_is_deterministic_for_fixed_now() {
        let client = build_test_client();
        let now = fixed_now();

        let a = client
            .get_object()
            .bucket("examplebucket")
            .key("test.txt")
            .presigned(3600, now)
            .expect("presigned");
        let b = client
            .get_object()
            .bucket("examplebucket")
            .key("test.txt")
            .presigned(3600, now)
            .expect("presigned");

        assert_eq!(a.url, b.url);
        assert_eq!(a.headers, b.headers);
        assert_eq!(a.body, b.body);
    }

    /// `UNIX_EPOCH` 前の時刻は `Error::InvalidInput` で弾く
    #[test]
    fn test_build_request_rejects_pre_epoch_now() {
        let client = build_test_client();
        let pre_epoch = UNIX_EPOCH - Duration::from_secs(1);
        let result = client
            .get_object()
            .bucket("examplebucket")
            .key("test.txt")
            .build_request(pre_epoch);
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    /// SSE-C を指定した `build_request` に標準 SSE-C ヘッダーが含まれる
    ///
    /// ヘルパー `add_sse_c_headers` への置換が正しく行われていることを、
    /// API 経由の出力で検証する。MD5 は "0123456789abcdef" の MD5 を
    /// Base64 で表現した値。
    #[test]
    fn test_get_object_build_request_sse_c_headers() {
        let client = build_test_client();
        let request = client
            .get_object()
            .bucket("examplebucket")
            .key("test.txt")
            .sse_customer_algorithm("AES256")
            .sse_customer_key("MDEyMzQ1Njc4OWFiY2RlZg==")
            .build_request(fixed_now())
            .expect("build_request");

        assert!(request.headers.iter().any(|(k, v)| {
            k == "x-amz-server-side-encryption-customer-algorithm" && v == "AES256"
        }));
        assert!(request.headers.iter().any(|(k, v)| {
            k == "x-amz-server-side-encryption-customer-key" && v == "MDEyMzQ1Njc4OWFiY2RlZg=="
        }));
        assert!(request.headers.iter().any(|(k, v)| {
            k == "x-amz-server-side-encryption-customer-key-md5" && v == "QDKvjWEDUSOQbljgZxQMxQ=="
        }));
    }

    /// CopyObject の `build_request` に標準 SSE-C とコピー元 SSE-C の両方のヘッダーが含まれる
    ///
    /// 標準 (`x-amz-server-side-encryption-customer-*`) とコピー元
    /// (`x-amz-copy-source-server-side-encryption-customer-*`) でヘッダー名が
    /// 正しく切り替わることを検証する。
    #[test]
    fn test_copy_object_build_request_sse_c_headers() {
        let client = build_test_client();
        let request = client
            .copy_object()
            .bucket("destbucket")
            .key("dest.txt")
            .copy_source("srcbucket/src.txt")
            .sse_customer_algorithm("AES256")
            .sse_customer_key("MDEyMzQ1Njc4OWFiY2RlZg==")
            .copy_source_sse_customer_algorithm("AES256")
            .copy_source_sse_customer_key("MDEyMzQ1Njc4OWFiY2RlZg==")
            .build_request(fixed_now())
            .expect("build_request");

        // コピー先 (標準) SSE-C ヘッダー
        assert!(request.headers.iter().any(|(k, v)| {
            k == "x-amz-server-side-encryption-customer-algorithm" && v == "AES256"
        }));
        assert!(request.headers.iter().any(|(k, v)| {
            k == "x-amz-server-side-encryption-customer-key" && v == "MDEyMzQ1Njc4OWFiY2RlZg=="
        }));
        assert!(request.headers.iter().any(|(k, v)| {
            k == "x-amz-server-side-encryption-customer-key-md5" && v == "QDKvjWEDUSOQbljgZxQMxQ=="
        }));
        // コピー元 SSE-C ヘッダー
        assert!(request.headers.iter().any(|(k, v)| {
            k == "x-amz-copy-source-server-side-encryption-customer-algorithm" && v == "AES256"
        }));
        assert!(request.headers.iter().any(|(k, v)| {
            k == "x-amz-copy-source-server-side-encryption-customer-key"
                && v == "MDEyMzQ1Njc4OWFiY2RlZg=="
        }));
        assert!(request.headers.iter().any(|(k, v)| {
            k == "x-amz-copy-source-server-side-encryption-customer-key-md5"
                && v == "QDKvjWEDUSOQbljgZxQMxQ=="
        }));
        // コピー元指定が x-amz-copy-source ヘッダーに正規化されて含まれること
        assert!(
            request
                .headers
                .iter()
                .any(|(k, v)| { k == "x-amz-copy-source" && v == "/srcbucket/src.txt" })
        );
    }

    /// SSE-C を指定した `presigned` の署名対象ヘッダーに SSE-C ヘッダーが含まれる
    ///
    /// presigned URL は SSE-C ヘッダーを X-Amz-SignedHeaders に含めないと
    /// S3 がリクエストを拒否するため、署名対象への包含を検証する。
    #[test]
    fn test_get_object_presigned_sse_c_headers() {
        let client = build_test_client();
        let presigned = client
            .get_object()
            .bucket("examplebucket")
            .key("test.txt")
            .sse_customer_algorithm("AES256")
            .sse_customer_key("MDEyMzQ1Njc4OWFiY2RlZg==")
            .presigned(3600, fixed_now())
            .expect("presigned");

        // URL の X-Amz-SignedHeaders に SSE-C ヘッダーが含まれること
        let signed_headers = presigned
            .url
            .split("X-Amz-SignedHeaders=")
            .nth(1)
            .expect("X-Amz-SignedHeaders が含まれること")
            .split('&')
            .next()
            .expect("SignedHeaders 値が取得できること");
        assert!(signed_headers.contains("x-amz-server-side-encryption-customer-algorithm"));
        assert!(signed_headers.contains("x-amz-server-side-encryption-customer-key"));
        assert!(signed_headers.contains("x-amz-server-side-encryption-customer-key-md5"));
        // リクエスト時に付与が必要なヘッダーにも含まれること
        assert!(
            presigned
                .headers
                .iter()
                .any(|(k, _)| k == "x-amz-server-side-encryption-customer-algorithm")
        );
        assert!(
            presigned
                .headers
                .iter()
                .any(|(k, _)| k == "x-amz-server-side-encryption-customer-key")
        );
        assert!(
            presigned
                .headers
                .iter()
                .any(|(k, _)| k == "x-amz-server-side-encryption-customer-key-md5")
        );
    }
}
