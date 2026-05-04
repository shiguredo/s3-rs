// -------------------------------------------------------
// アップロードパラメータ
// -------------------------------------------------------

/// アップロード時の追加パラメータ (SSE, ACL, メタデータ等)
#[derive(Clone, Default)]
pub(crate) struct UploadParams {
    pub(crate) sse: Option<String>,
    pub(crate) sse_kms_key_id: Option<String>,
    pub(crate) sse_c: Option<String>,
    pub(crate) sse_c_key: Option<String>,
    pub(crate) sse_c_copy_source: Option<String>,
    pub(crate) sse_c_copy_source_key: Option<String>,
    pub(crate) acl: Option<String>,
    pub(crate) cache_control: Option<String>,
    pub(crate) content_disposition: Option<String>,
    pub(crate) content_encoding: Option<String>,
    pub(crate) content_language: Option<String>,
    pub(crate) expires: Option<String>,
    pub(crate) metadata_directive: Option<String>,
    pub(crate) metadata: Vec<(String, String)>,
    pub(crate) storage_class: Option<String>,
    pub(crate) checksum_algorithm: Option<String>,
}

impl UploadParams {
    /// PutObject ビルダーにパラメータを適用する
    pub(crate) fn apply_to_put<'a>(
        &self,
        mut builder: shiguredo_s3::api::PutObjectFluentBuilder<'a>,
    ) -> shiguredo_s3::api::PutObjectFluentBuilder<'a> {
        if let Some(ref v) = self.acl {
            builder = builder.acl(shiguredo_s3::ObjectCannedAcl::from(v.as_str()));
        }
        if let Some(ref v) = self.cache_control {
            builder = builder.cache_control(v);
        }
        if let Some(ref v) = self.content_disposition {
            builder = builder.content_disposition(v);
        }
        if let Some(ref v) = self.content_encoding {
            builder = builder.content_encoding(v);
        }
        if let Some(ref v) = self.content_language {
            builder = builder.content_language(v);
        }
        if let Some(ref v) = self.expires {
            builder = builder.expires(v);
        }
        for (k, v) in &self.metadata {
            builder = builder.metadata(k, v);
        }
        if let Some(ref v) = self.storage_class {
            builder = builder.storage_class(shiguredo_s3::StorageClass::from(v.as_str()));
        }
        if let Some(ref v) = self.sse {
            builder = builder
                .server_side_encryption(shiguredo_s3::ServerSideEncryption::from(v.as_str()));
        }
        if let Some(ref v) = self.sse_kms_key_id {
            builder = builder.ssekms_key_id(v);
        }
        if let Some(ref v) = self.sse_c {
            builder = builder.sse_customer_algorithm(v);
        }
        if let Some(ref key) = self.sse_c_key {
            builder = builder.sse_customer_key(key);
        }
        if let Some(ref v) = self.checksum_algorithm {
            builder = builder.checksum_algorithm(shiguredo_s3::ChecksumAlgorithm::from(v.as_str()));
        }
        builder
    }

    /// CreateMultipartUpload ビルダーにパラメータを適用する
    pub(crate) fn apply_to_create_multipart<'a>(
        &self,
        mut builder: shiguredo_s3::api::CreateMultipartUploadFluentBuilder<'a>,
    ) -> shiguredo_s3::api::CreateMultipartUploadFluentBuilder<'a> {
        if let Some(ref v) = self.acl {
            builder = builder.acl(shiguredo_s3::ObjectCannedAcl::from(v.as_str()));
        }
        if let Some(ref v) = self.storage_class {
            builder = builder.storage_class(shiguredo_s3::StorageClass::from(v.as_str()));
        }
        if let Some(ref v) = self.cache_control {
            builder = builder.cache_control(v);
        }
        if let Some(ref v) = self.content_disposition {
            builder = builder.content_disposition(v);
        }
        if let Some(ref v) = self.content_encoding {
            builder = builder.content_encoding(v);
        }
        if let Some(ref v) = self.content_language {
            builder = builder.content_language(v);
        }
        if let Some(ref v) = self.expires {
            builder = builder.expires(v);
        }
        for (k, v) in &self.metadata {
            builder = builder.metadata(k, v);
        }
        if let Some(ref v) = self.sse {
            builder = builder
                .server_side_encryption(shiguredo_s3::ServerSideEncryption::from(v.as_str()));
        }
        if let Some(ref v) = self.sse_kms_key_id {
            builder = builder.ssekms_key_id(v);
        }
        if let Some(ref v) = self.sse_c {
            builder = builder.sse_customer_algorithm(v);
        }
        if let Some(ref key) = self.sse_c_key {
            builder = builder.sse_customer_key(key);
        }
        builder
    }

    /// UploadPart ビルダーにパラメータを適用する
    pub(crate) fn apply_to_upload_part<'a>(
        &self,
        mut builder: shiguredo_s3::api::UploadPartFluentBuilder<'a>,
    ) -> shiguredo_s3::api::UploadPartFluentBuilder<'a> {
        if let Some(ref v) = self.sse_c {
            builder = builder.sse_customer_algorithm(v);
        }
        if let Some(ref key) = self.sse_c_key {
            builder = builder.sse_customer_key(key);
        }
        if let Some(ref v) = self.checksum_algorithm {
            builder = builder.checksum_algorithm(shiguredo_s3::ChecksumAlgorithm::from(v.as_str()));
        }
        builder
    }

    /// GetObject ビルダーに SSE-C パラメータを適用する
    pub(crate) fn apply_to_get<'a>(
        &self,
        mut builder: shiguredo_s3::api::GetObjectFluentBuilder<'a>,
    ) -> shiguredo_s3::api::GetObjectFluentBuilder<'a> {
        if let Some(ref v) = self.sse_c {
            builder = builder.sse_customer_algorithm(v);
        }
        if let Some(ref key) = self.sse_c_key {
            builder = builder.sse_customer_key(key);
        }
        builder
    }

    /// CopyObject ビルダーにパラメータを適用する
    pub(crate) fn apply_to_copy<'a>(
        &self,
        mut builder: shiguredo_s3::api::CopyObjectFluentBuilder<'a>,
    ) -> shiguredo_s3::api::CopyObjectFluentBuilder<'a> {
        if let Some(ref v) = self.acl {
            builder = builder.acl(shiguredo_s3::ObjectCannedAcl::from(v.as_str()));
        }
        if let Some(ref v) = self.storage_class {
            builder = builder.storage_class(shiguredo_s3::StorageClass::from(v.as_str()));
        }
        if let Some(ref v) = self.metadata_directive {
            builder = builder.metadata_directive(shiguredo_s3::MetadataDirective::from(v.as_str()));
        }
        if let Some(ref v) = self.cache_control {
            builder = builder.cache_control(v);
        }
        if let Some(ref v) = self.content_disposition {
            builder = builder.content_disposition(v);
        }
        if let Some(ref v) = self.content_encoding {
            builder = builder.content_encoding(v);
        }
        if let Some(ref v) = self.content_language {
            builder = builder.content_language(v);
        }
        if let Some(ref v) = self.expires {
            builder = builder.expires(v);
        }
        for (k, v) in &self.metadata {
            builder = builder.metadata(k, v);
        }
        // コピー先の SSE
        if let Some(ref v) = self.sse {
            builder = builder
                .server_side_encryption(shiguredo_s3::ServerSideEncryption::from(v.as_str()));
        }
        if let Some(ref v) = self.sse_kms_key_id {
            builder = builder.ssekms_key_id(v);
        }
        if let Some(ref v) = self.sse_c {
            builder = builder.sse_customer_algorithm(v);
        }
        if let Some(ref key) = self.sse_c_key {
            builder = builder.sse_customer_key(key);
        }
        // コピー元の SSE-C
        if let Some(ref v) = self.sse_c_copy_source {
            builder = builder.copy_source_sse_customer_algorithm(v);
        }
        if let Some(ref key) = self.sse_c_copy_source_key {
            builder = builder.copy_source_sse_customer_key(key);
        }
        builder
    }
}
