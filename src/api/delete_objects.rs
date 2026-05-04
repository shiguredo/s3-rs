//! DeleteObjects API
//!
//! 1 回のリクエストで複数のオブジェクトを一括削除する。
//! 最大 1,000 個のオブジェクトを指定できる。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjects.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{
    ChecksumAlgorithm, DeleteError, DeleteObjectsOutput, DeletedObject, ObjectIdentifier,
};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct DeleteObjectsFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    objects: Vec<ObjectIdentifier>,
    quiet: bool,
    checksum_algorithm: Option<ChecksumAlgorithm>,
}

impl<'a> DeleteObjectsFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            objects: Vec::new(),
            quiet: false,
            checksum_algorithm: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// 削除対象オブジェクトを追加する
    pub fn object(mut self, object: ObjectIdentifier) -> Self {
        self.objects.push(object);
        self
    }

    /// quiet モードを設定する (true の場合、エラーのみレスポンスに含まれる)
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
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

        if self.objects.is_empty() {
            return Err(Error::InvalidInput(
                "at least one object is required".to_string(),
            ));
        }

        if self.objects.len() > 1000 {
            return Err(Error::InvalidInput(
                "at most 1000 objects are allowed per request".to_string(),
            ));
        }

        let xml_body = build_delete_objects_xml(&self.objects, self.quiet);
        let content_md5 = base64_md5(xml_body.as_bytes());

        let mut extra_headers: Vec<(&str, &str)> = vec![
            ("content-type", "application/xml"),
            ("content-md5", content_md5.as_str()),
        ];

        let computed_checksum;
        if let Some(ref algorithm) = self.checksum_algorithm {
            extra_headers.push(("x-amz-checksum-algorithm", algorithm.as_str()));
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

        let deleted = extract_xml_deleted_objects(body_text);
        let errors = extract_xml_delete_errors(body_text);

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

fn build_delete_objects_xml(objects: &[ObjectIdentifier], quiet: bool) -> String {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("Delete", crate::xml::S3_NS);
    if quiet {
        w.element("Quiet", "true");
    }
    for obj in objects {
        w.start("Object");
        w.element("Key", &obj.key);
        if let Some(ref version_id) = obj.version_id {
            w.element("VersionId", version_id);
        }
        w.end();
    }
    w.end();
    w.finish()
}

fn extract_xml_deleted_objects(text: &str) -> Vec<DeletedObject> {
    let mut deleted = Vec::new();
    crate::xml::for_each_element(text, "Deleted", |elem| {
        deleted.push(DeletedObject {
            key: elem.get("Key").map(String::from),
            version_id: elem.get("VersionId").map(String::from),
            delete_marker: elem.get_parsed::<bool>("DeleteMarker"),
            delete_marker_version_id: elem.get("DeleteMarkerVersionId").map(String::from),
        });
    });
    deleted
}

fn extract_xml_delete_errors(text: &str) -> Vec<DeleteError> {
    let mut errors = Vec::new();
    crate::xml::for_each_element(text, "Error", |elem| {
        errors.push(DeleteError {
            key: elem.get("Key").map(String::from),
            code: elem.get("Code").map(String::from),
            message: elem.get("Message").map(String::from),
        });
    });
    errors
}
