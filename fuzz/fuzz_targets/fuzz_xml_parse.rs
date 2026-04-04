#![no_main]

//! XML レスポンスパースの fuzz ターゲット
//!
//! 任意のバイト列を S3 レスポンスボディとして各 parse_response に渡し、
//! パニックしないことを検証する。

use libfuzzer_sys::fuzz_target;
use shiguredo_s3::api::{
    CompleteMultipartUploadFluentBuilder, CopyObjectFluentBuilder,
    CreateMultipartUploadFluentBuilder, DeleteObjectsFluentBuilder,
    GetBucketEncryptionFluentBuilder, GetBucketLifecycleConfigurationFluentBuilder,
    GetBucketNotificationConfigurationFluentBuilder, GetBucketTaggingFluentBuilder,
    GetBucketWebsiteFluentBuilder,
    GetBucketVersioningFluentBuilder, GetPublicAccessBlockFluentBuilder,
    ListBucketsFluentBuilder, ListMultipartUploadsFluentBuilder, ListObjectsV2FluentBuilder,
    ListPartsFluentBuilder, S3Response,
};

fuzz_target!(|data: &[u8]| {
    let response = S3Response {
        status_code: 200,
        headers: vec![
            ("content-type".to_string(), "application/xml".to_string()),
        ],
        body: data.to_vec(),
    };

    // 各 XML パーサーにデータを投入する（パニックしなければ OK）
    let _ = ListObjectsV2FluentBuilder::parse_response(&response);
    let _ = ListBucketsFluentBuilder::parse_response(&response);
    let _ = ListMultipartUploadsFluentBuilder::parse_response(&response);
    let _ = ListPartsFluentBuilder::parse_response(&response);
    let _ = CreateMultipartUploadFluentBuilder::parse_response(&response);
    let _ = DeleteObjectsFluentBuilder::parse_response(&response);
    let _ = GetBucketTaggingFluentBuilder::parse_response(&response);
    let _ = GetBucketVersioningFluentBuilder::parse_response(&response);
    let _ = GetPublicAccessBlockFluentBuilder::parse_response(&response);
    let _ = CompleteMultipartUploadFluentBuilder::parse_response(&response);
    let _ = CopyObjectFluentBuilder::parse_response(&response);
    let _ = GetBucketEncryptionFluentBuilder::parse_response(&response);
    let _ = GetBucketLifecycleConfigurationFluentBuilder::parse_response(&response);
    let _ = GetBucketNotificationConfigurationFluentBuilder::parse_response(&response);
    let _ = GetBucketWebsiteFluentBuilder::parse_response(&response);
});
