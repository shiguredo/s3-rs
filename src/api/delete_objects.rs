//! DeleteObjects API
//!
//! 1 回のリクエストで複数のオブジェクトを一括削除する。
//! 最大 1,000 個のオブジェクトを指定できる。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjects.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{
    ChecksumAlgorithm, Delete, DeleteError, DeleteObjectsOutput, DeletedObject, ObjectIdentifier,
};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct DeleteObjectsFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    delete: Option<Delete>,
    checksum_algorithm: Option<ChecksumAlgorithm>,
}

impl<'a> DeleteObjectsFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            delete: None,
            checksum_algorithm: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// `Delete` (削除対象オブジェクトの一覧と quiet 設定) を指定する
    ///
    /// aws-sdk-rust の `DeleteObjectsFluentBuilder::delete(Delete)` と同じ。
    pub fn delete(mut self, delete: Delete) -> Self {
        self.delete = Some(delete);
        self
    }

    /// `Delete` を Option で設定する (`set_*` バリアント)
    pub fn set_delete(mut self, delete: Option<Delete>) -> Self {
        self.delete = delete;
        self
    }

    /// チェックサムアルゴリズムを指定する (CRC32, CRC32C, SHA1, SHA256, CRC64NVME)
    pub fn checksum_algorithm(mut self, input: ChecksumAlgorithm) -> Self {
        self.checksum_algorithm = Some(input);
        self
    }

    pub fn set_checksum_algorithm(mut self, input: Option<ChecksumAlgorithm>) -> Self {
        self.checksum_algorithm = input;
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let delete = self
            .delete
            .as_ref()
            .ok_or_else(|| Error::InvalidInput("delete is required".to_string()))?;

        if delete.objects.is_empty() {
            return Err(Error::InvalidInput(
                "at least one object is required".to_string(),
            ));
        }

        if delete.objects.len() > 1000 {
            return Err(Error::InvalidInput(
                "at most 1000 objects are allowed per request".to_string(),
            ));
        }

        let xml_body = build_delete_objects_xml(&delete.objects, delete.quiet.unwrap_or(false))?;
        let content_md5 = base64_md5(xml_body.as_bytes());

        let mut extra_headers: Vec<(&str, &str)> = vec![
            ("content-type", "application/xml"),
            ("content-md5", content_md5.as_str()),
        ];

        let computed_checksum;
        if let Some(ref algorithm) = self.checksum_algorithm {
            extra_headers.push(("x-amz-sdk-checksum-algorithm", algorithm.as_str()));
            let header_name = crate::checksum::header_name(algorithm)?;
            computed_checksum = crate::checksum::compute_checksum(algorithm, xml_body.as_bytes())?;
            extra_headers.push((header_name, &computed_checksum));
        }

        let query_params = [("delete", "")];

        build_signed_request(
            &self.client.config_ref(),
            "POST",
            bucket,
            "",
            &extra_headers,
            xml_body.as_bytes(),
            Some(&query_params),
            now,
        )
    }

    pub fn parse_response(response: &super::S3Response) -> Result<DeleteObjectsOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;

        let deleted = extract_xml_deleted_objects(body_text)?;
        let errors = extract_xml_delete_errors(body_text)?;

        Ok(DeleteObjectsOutput {
            deleted: if deleted.is_empty() {
                None
            } else {
                Some(deleted)
            },
            errors: if errors.is_empty() {
                None
            } else {
                Some(errors)
            },
        })
    }
}

fn build_delete_objects_xml(objects: &[ObjectIdentifier], quiet: bool) -> Result<String, Error> {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("Delete", crate::xml::S3_NS);
    if quiet {
        w.element("Quiet", "true")?;
    }
    for obj in objects {
        w.start("Object");
        w.element("Key", &obj.key)?;
        if let Some(ref version_id) = obj.version_id {
            w.element("VersionId", version_id)?;
        }
        if let Some(ref e_tag) = obj.e_tag {
            w.element("ETag", e_tag)?;
        }
        if let Some(t) = obj.last_modified_time {
            // S3 仕様では LastModifiedTime は IMF-fixdate (HTTP-date) 形式で送る
            // https://docs.aws.amazon.com/AmazonS3/latest/API/API_ObjectIdentifier.html
            let formatted = crate::datetime::format_imf_fixdate(t)?;
            w.element("LastModifiedTime", &formatted)?;
        }
        if let Some(size) = obj.size {
            w.element("Size", &size.to_string())?;
        }
        w.end();
    }
    w.end();
    Ok(w.finish())
}

fn extract_xml_deleted_objects(text: &str) -> Result<Vec<DeletedObject>, Error> {
    let mut deleted = Vec::new();
    crate::xml::for_each_element(text, "Deleted", |elem| {
        deleted.push(DeletedObject {
            key: elem.get("Key").map(String::from),
            version_id: elem.get("VersionId").map(String::from),
            delete_marker: elem.get_parsed::<bool>("DeleteMarker"),
            delete_marker_version_id: elem.get("DeleteMarkerVersionId").map(String::from),
        });
    })?;
    Ok(deleted)
}

fn extract_xml_delete_errors(text: &str) -> Result<Vec<DeleteError>, Error> {
    let mut errors = Vec::new();
    crate::xml::for_each_element(text, "Error", |elem| {
        errors.push(DeleteError {
            key: elem.get("Key").map(String::from),
            code: elem.get("Code").map(String::from),
            message: elem.get("Message").map(String::from),
        });
    })?;
    Ok(errors)
}
