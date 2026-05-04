//! CompleteMultipartUpload API
//!
//! アップロード済みのパートを結合してマルチパートアップロードを完了する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{CompleteMultipartUploadOutput, CompletedMultipartUpload};

use super::{
    S3Request, build_presigned_url, build_signed_request, check_body_error, parse_error_response,
    required, validate_presign_expires,
};

pub struct CompleteMultipartUploadFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    key: Option<String>,
    upload_id: Option<String>,
    multipart_upload: Option<CompletedMultipartUpload>,
    /// SSE-C アルゴリズム (AES256)
    sse_customer_algorithm: Option<String>,
    /// SSE-C キー (Base64)
    sse_customer_key: Option<String>,
    /// 条件付き書き込み: ETag が一致する場合のみ完了する
    if_match: Option<String>,
    /// 条件付き書き込み: オブジェクトが存在しない場合のみ完了する ("*")
    if_none_match: Option<String>,
}

impl<'a> CompleteMultipartUploadFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            upload_id: None,
            multipart_upload: None,
            sse_customer_algorithm: None,
            sse_customer_key: None,
            if_match: None,
            if_none_match: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn upload_id(mut self, upload_id: impl Into<String>) -> Self {
        self.upload_id = Some(upload_id.into());
        self
    }

    pub fn multipart_upload(mut self, multipart_upload: CompletedMultipartUpload) -> Self {
        self.multipart_upload = Some(multipart_upload);
        self
    }

    /// SSE-C アルゴリズムを指定する (AES256)
    pub fn sse_customer_algorithm(mut self, algorithm: impl Into<String>) -> Self {
        self.sse_customer_algorithm = Some(algorithm.into());
        self
    }

    /// SSE-C キーを指定する (Base64 エンコード)
    ///
    /// MD5 はキーから自動計算される。
    pub fn sse_customer_key(mut self, key: impl Into<String>) -> Self {
        self.sse_customer_key = Some(key.into());
        self
    }

    /// ETag が一致する場合のみ完了する (楽観的ロック)
    pub fn if_match(mut self, e_tag: impl Into<String>) -> Self {
        self.if_match = Some(e_tag.into());
        self
    }

    /// オブジェクトが存在しない場合のみ完了する ("*" を指定)
    pub fn if_none_match(mut self, value: impl Into<String>) -> Self {
        self.if_none_match = Some(value.into());
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;
        let upload_id = required(self.upload_id.as_deref(), "upload_id")?;

        if let Some(ref upload) = self.multipart_upload
            && let Some(ref parts) = upload.parts
        {
            let mut prev_part_number = 0i32;
            for (i, part) in parts.iter().enumerate() {
                let pn = part.part_number.ok_or_else(|| {
                    Error::InvalidInput(format!("part[{i}] is missing part_number"))
                })?;
                if part.e_tag.is_none() {
                    return Err(Error::InvalidInput(format!("part[{i}] is missing e_tag")));
                }
                if pn <= prev_part_number {
                    return Err(Error::InvalidInput(
                        "parts must be in ascending order of part_number".to_string(),
                    ));
                }
                prev_part_number = pn;
            }
        }

        let xml_body = build_complete_multipart_xml(&self.multipart_upload);
        let query_params = [("uploadId", upload_id)];
        let mut extra_headers: Vec<(&str, &str)> = vec![("content-type", "application/xml")];
        if let Some(ref v) = self.if_match {
            extra_headers.push(("if-match", v.as_str()));
        }
        if let Some(ref v) = self.if_none_match {
            extra_headers.push(("if-none-match", v.as_str()));
        }

        if let Some(ref v) = self.sse_customer_algorithm {
            extra_headers.push((
                "x-amz-server-side-encryption-customer-algorithm",
                v.as_str(),
            ));
        }
        // SSE-C キーが指定されている場合、MD5 を自動計算する
        let computed_key_md5;
        if let Some(ref v) = self.sse_customer_key {
            extra_headers.push(("x-amz-server-side-encryption-customer-key", v.as_str()));
            computed_key_md5 = super::compute_sse_c_key_md5(v)?;
            extra_headers.push((
                "x-amz-server-side-encryption-customer-key-md5",
                &computed_key_md5,
            ));
        }

        build_signed_request(
            &self.client.config_ref(),
            "POST",
            bucket,
            key,
            &extra_headers,
            xml_body.as_bytes(),
            Some(&query_params),
            now,
        )
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<CompleteMultipartUploadOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        // S3 は 200 OK でもボディに <Error> を返すことがある
        check_body_error(response)?;

        let body_text = std::str::from_utf8(&response.body).ok();

        Ok(CompleteMultipartUploadOutput {
            location: body_text.and_then(|t| crate::xml::extract_element(t, "Location")),
            bucket: body_text.and_then(|t| crate::xml::extract_element(t, "Bucket")),
            key: body_text.and_then(|t| crate::xml::extract_element(t, "Key")),
            e_tag: body_text.and_then(|t| crate::xml::extract_element(t, "ETag")),
            version_id: response.get_header("x-amz-version-id").map(String::from),
        })
    }

    /// Presigned リクエストを生成する (Sans I/O)
    ///
    /// CompleteMultipartUpload は POST ボディに completed parts の XML が必須のため、
    /// `PresignedRequest::body` に XML を含めて返す。
    pub fn presigned(
        self,
        expires_in_secs: u64,
        now: std::time::SystemTime,
    ) -> Result<super::PresignedRequest, Error> {
        validate_presign_expires(expires_in_secs)?;
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;
        let upload_id = required(self.upload_id.as_deref(), "upload_id")?;
        let xml_body = build_complete_multipart_xml(&self.multipart_upload);
        let url = build_presigned_url(
            &self.client.config_ref(),
            "POST",
            bucket,
            key,
            expires_in_secs,
            &[("uploadId", upload_id)],
            &[],
            now,
        )?;
        Ok(super::PresignedRequest {
            url,
            method: "POST".to_string(),
            headers: Vec::new(),
            body: xml_body.into_bytes(),
        })
    }
}

fn build_complete_multipart_xml(multipart_upload: &Option<CompletedMultipartUpload>) -> String {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("CompleteMultipartUpload", crate::xml::S3_NS);

    if let Some(upload) = multipart_upload
        && let Some(parts) = &upload.parts
    {
        for part in parts {
            w.start("Part");
            if let Some(part_number) = part.part_number {
                w.element("PartNumber", &part_number.to_string());
            }
            if let Some(ref e_tag) = part.e_tag {
                w.element("ETag", e_tag);
            }
            w.end();
        }
    }

    w.end();
    w.finish()
}
