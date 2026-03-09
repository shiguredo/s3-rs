use std::fmt;

/// S3 エラーコード定数
pub mod s3_error_code {
    pub const NO_SUCH_KEY: &str = "NoSuchKey";
    pub const NO_SUCH_BUCKET: &str = "NoSuchBucket";
    pub const ACCESS_DENIED: &str = "AccessDenied";
    pub const NO_SUCH_UPLOAD: &str = "NoSuchUpload";
    pub const ENTITY_TOO_LARGE: &str = "EntityTooLarge";
    pub const ENTITY_TOO_SMALL: &str = "EntityTooSmall";
    pub const INVALID_PART: &str = "InvalidPart";
    pub const INVALID_PART_ORDER: &str = "InvalidPartOrder";
    pub const INVALID_RANGE: &str = "InvalidRange";
    pub const INVALID_ARGUMENT: &str = "InvalidArgument";
    pub const INVALID_BUCKET_NAME: &str = "InvalidBucketName";
    pub const MALFORMED_XML: &str = "MalformedXML";
    pub const SIGNATURE_DOES_NOT_MATCH: &str = "SignatureDoesNotMatch";
    pub const INVALID_ACCESS_KEY_ID: &str = "InvalidAccessKeyId";
    pub const REQUEST_TIME_TOO_SKEWED: &str = "RequestTimeTooSkewed";
    pub const SLOW_DOWN: &str = "SlowDown";
    pub const SERVICE_UNAVAILABLE: &str = "ServiceUnavailable";
    pub const INTERNAL_ERROR: &str = "InternalError";
    pub const PRECONDITION_FAILED: &str = "PreconditionFailed";
}

#[derive(Debug)]
pub enum Error {
    /// 不正なレスポンス
    InvalidResponse(String),
    /// 不正な入力パラメータ
    InvalidInput(String),
    /// 304 Not Modified (条件付きリクエストでオブジェクトが変更されていない)
    NotModified,
    /// 412 Precondition Failed (条件付きリクエストの前提条件が失敗)
    PreconditionFailed,
    /// S3 エラーレスポンス
    S3 {
        status_code: u16,
        code: String,
        message: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidResponse(msg) => write!(f, "invalid response: {msg}"),
            Error::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            Error::NotModified => write!(f, "not modified (304)"),
            Error::PreconditionFailed => write!(f, "precondition failed (412)"),
            Error::S3 {
                status_code,
                code,
                message,
            } => {
                write!(f, "S3 error ({status_code}): {code} - {message}")
            }
        }
    }
}

impl std::error::Error for Error {}
