//! Error types for Hafiz

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    // Bucket Errors
    #[error("The specified bucket does not exist")]
    NoSuchBucket,

    #[error("The specified bucket does not exist: {0}")]
    NoSuchBucketNamed(String),

    #[error("The requested bucket name is not available")]
    BucketAlreadyExists,

    #[error("The bucket you tried to delete is not empty")]
    BucketNotEmpty,

    #[error("The bucket does not have a policy")]
    NoSuchBucketPolicy,

    // Object Errors
    #[error("The specified key does not exist")]
    NoSuchKey,

    #[error("The specified key does not exist: {0}")]
    NoSuchKeyNamed(String),

    #[error("The specified multipart upload does not exist")]
    NoSuchUpload,

    #[error("The lifecycle configuration does not exist")]
    NoSuchLifecycleConfiguration,

    #[error("Invalid part: {0}")]
    InvalidPart(String),

    #[error("Object is too large")]
    EntityTooLarge,

    // Access Errors
    #[error("Access Denied")]
    AccessDenied,

    #[error("The AWS access key ID you provided does not exist")]
    InvalidAccessKeyId,

    #[error("The request signature does not match")]
    SignatureDoesNotMatch,

    #[error("Request has expired")]
    ExpiredPresignedRequest,

    // Policy and ACL Errors
    #[error("Malformed policy document: {0}")]
    MalformedPolicy(String),

    #[error("Malformed ACL: {0}")]
    MalformedACL(String),

    // Validation Errors
    #[error("Invalid bucket name: {0}")]
    InvalidBucketName(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Malformed XML: {0}")]
    MalformedXML(String),

    #[error("Missing required header: {0}")]
    MissingHeader(String),

    #[error("Invalid range: {0}")]
    InvalidRange(String),

    // Storage Errors
    #[error("Storage backend error: {0}")]
    StorageError(String),

    // Database Errors
    #[error("Database error: {0}")]
    DatabaseError(String),

    // Internal Errors
    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Error::NoSuchBucket | Error::NoSuchBucketNamed(_) => "NoSuchBucket",
            Error::BucketAlreadyExists => "BucketAlreadyExists",
            Error::BucketNotEmpty => "BucketNotEmpty",
            Error::NoSuchBucketPolicy => "NoSuchBucketPolicy",
            Error::NoSuchKey | Error::NoSuchKeyNamed(_) => "NoSuchKey",
            Error::NoSuchUpload => "NoSuchUpload",
            Error::NoSuchLifecycleConfiguration => "NoSuchLifecycleConfiguration",
            Error::InvalidPart(_) => "InvalidPart",
            Error::EntityTooLarge => "EntityTooLarge",
            Error::AccessDenied => "AccessDenied",
            Error::InvalidAccessKeyId => "InvalidAccessKeyId",
            Error::SignatureDoesNotMatch => "SignatureDoesNotMatch",
            Error::ExpiredPresignedRequest => "AccessDenied",
            Error::MalformedPolicy(_) => "MalformedPolicy",
            Error::MalformedACL(_) => "MalformedACLError",
            Error::InvalidBucketName(_) => "InvalidBucketName",
            Error::InvalidArgument(_) => "InvalidArgument",
            Error::InvalidRequest(_) => "InvalidRequest",
            Error::MalformedXML(_) => "MalformedXMLDocument",
            Error::MissingHeader(_) => "MissingSecurityHeader",
            Error::InvalidRange(_) => "InvalidRange",
            Error::StorageError(_) => "InternalError",
            Error::DatabaseError(_) => "InternalError",
            Error::InternalError(_) => "InternalError",
            Error::NotImplemented(_) => "NotImplemented",
            Error::Io(_) => "InternalError",
            Error::Other(_) => "InternalError",
        }
    }

    pub fn http_status(&self) -> u16 {
        match self {
            Error::InvalidBucketName(_)
            | Error::InvalidArgument(_)
            | Error::InvalidRequest(_)
            | Error::MalformedXML(_)
            | Error::MalformedPolicy(_)
            | Error::MalformedACL(_)
            | Error::MissingHeader(_)
            | Error::InvalidPart(_)
            | Error::EntityTooLarge => 400,

            Error::AccessDenied
            | Error::InvalidAccessKeyId
            | Error::SignatureDoesNotMatch
            | Error::ExpiredPresignedRequest => 403,

            Error::NoSuchBucket
            | Error::NoSuchBucketNamed(_)
            | Error::NoSuchKey
            | Error::NoSuchKeyNamed(_)
            | Error::NoSuchUpload
            | Error::NoSuchLifecycleConfiguration
            | Error::NoSuchBucketPolicy => 404,

            Error::BucketAlreadyExists | Error::BucketNotEmpty => 409,

            Error::InvalidRange(_) => 416,

            Error::NotImplemented(_) => 501,

            _ => 500,
        }
    }
}

/// S3 Error Response
#[derive(Debug, Clone)]
pub struct S3Error {
    pub code: String,
    pub message: String,
    pub resource: Option<String>,
    pub request_id: String,
}

impl From<Error> for S3Error {
    fn from(err: Error) -> Self {
        S3Error {
            code: err.code().to_string(),
            message: err.to_string(),
            resource: None,
            request_id: String::new(),
        }
    }
}

impl S3Error {
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = request_id.into();
        self
    }

    pub fn to_xml(&self) -> String {
        let resource = self.resource.as_deref().unwrap_or("");
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Error>
<Code>{}</Code>
<Message>{}</Message>
<Resource>{}</Resource>
<RequestId>{}</RequestId>
</Error>"#,
            xml_escape(&self.code),
            xml_escape(&self.message),
            xml_escape(resource),
            xml_escape(&self.request_id)
        )
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
