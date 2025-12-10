//! XML response generation for S3 API

use hafiz_core::types::{BucketInfo, ListObjectsResult};
use hafiz_core::utils::format_s3_datetime;

/// Generate ListBuckets response XML
pub fn list_buckets_response(buckets: &[BucketInfo], owner_id: &str) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Owner>
    <ID>"#,
    );
    xml.push_str(owner_id);
    xml.push_str(
        r#"</ID>
    <DisplayName>"#,
    );
    xml.push_str(owner_id);
    xml.push_str(
        r#"</DisplayName>
  </Owner>
  <Buckets>"#,
    );

    for bucket in buckets {
        xml.push_str("\n    <Bucket>\n      <Name>");
        xml.push_str(&bucket.name);
        xml.push_str("</Name>\n      <CreationDate>");
        xml.push_str(&format_s3_datetime(&bucket.creation_date));
        xml.push_str("</CreationDate>\n    </Bucket>");
    }

    xml.push_str(
        r#"
  </Buckets>
</ListAllMyBucketsResult>"#,
    );

    xml
}

/// Generate ListObjects (v1) response XML
pub fn list_objects_response(result: &ListObjectsResult) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Name>"#,
    );
    xml.push_str(&result.name);
    xml.push_str("</Name>\n");

    if let Some(ref prefix) = result.prefix {
        xml.push_str("  <Prefix>");
        xml.push_str(prefix);
        xml.push_str("</Prefix>\n");
    } else {
        xml.push_str("  <Prefix></Prefix>\n");
    }

    if let Some(ref delimiter) = result.delimiter {
        xml.push_str("  <Delimiter>");
        xml.push_str(delimiter);
        xml.push_str("</Delimiter>\n");
    }

    xml.push_str(&format!("  <MaxKeys>{}</MaxKeys>\n", result.max_keys));
    xml.push_str(&format!(
        "  <IsTruncated>{}</IsTruncated>\n",
        result.is_truncated
    ));

    for obj in &result.contents {
        xml.push_str("  <Contents>\n");
        xml.push_str("    <Key>");
        xml.push_str(&xml_escape(&obj.key));
        xml.push_str("</Key>\n");
        xml.push_str("    <LastModified>");
        xml.push_str(&format_s3_datetime(&obj.last_modified));
        xml.push_str("</LastModified>\n");
        xml.push_str("    <ETag>\"");
        xml.push_str(&obj.etag);
        xml.push_str("\"</ETag>\n");
        xml.push_str(&format!("    <Size>{}</Size>\n", obj.size));
        xml.push_str("    <StorageClass>");
        xml.push_str(&obj.storage_class);
        xml.push_str("</StorageClass>\n");
        xml.push_str("  </Contents>\n");
    }

    for prefix in &result.common_prefixes {
        xml.push_str("  <CommonPrefixes>\n");
        xml.push_str("    <Prefix>");
        xml.push_str(&xml_escape(prefix));
        xml.push_str("</Prefix>\n");
        xml.push_str("  </CommonPrefixes>\n");
    }

    xml.push_str("</ListBucketResult>");
    xml
}

/// Generate ListObjectsV2 response XML
pub fn list_objects_v2_response(result: &ListObjectsResult) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Name>"#,
    );
    xml.push_str(&result.name);
    xml.push_str("</Name>\n");

    if let Some(ref prefix) = result.prefix {
        xml.push_str("  <Prefix>");
        xml.push_str(prefix);
        xml.push_str("</Prefix>\n");
    } else {
        xml.push_str("  <Prefix></Prefix>\n");
    }

    if let Some(ref delimiter) = result.delimiter {
        xml.push_str("  <Delimiter>");
        xml.push_str(delimiter);
        xml.push_str("</Delimiter>\n");
    }

    xml.push_str(&format!("  <MaxKeys>{}</MaxKeys>\n", result.max_keys));
    xml.push_str(&format!(
        "  <KeyCount>{}</KeyCount>\n",
        result.contents.len()
    ));
    xml.push_str(&format!(
        "  <IsTruncated>{}</IsTruncated>\n",
        result.is_truncated
    ));

    if let Some(ref token) = result.continuation_token {
        xml.push_str("  <ContinuationToken>");
        xml.push_str(token);
        xml.push_str("</ContinuationToken>\n");
    }

    if let Some(ref token) = result.next_continuation_token {
        xml.push_str("  <NextContinuationToken>");
        xml.push_str(token);
        xml.push_str("</NextContinuationToken>\n");
    }

    for obj in &result.contents {
        xml.push_str("  <Contents>\n");
        xml.push_str("    <Key>");
        xml.push_str(&xml_escape(&obj.key));
        xml.push_str("</Key>\n");
        xml.push_str("    <LastModified>");
        xml.push_str(&format_s3_datetime(&obj.last_modified));
        xml.push_str("</LastModified>\n");
        xml.push_str("    <ETag>\"");
        xml.push_str(&obj.etag);
        xml.push_str("\"</ETag>\n");
        xml.push_str(&format!("    <Size>{}</Size>\n", obj.size));
        xml.push_str("    <StorageClass>");
        xml.push_str(&obj.storage_class);
        xml.push_str("</StorageClass>\n");
        xml.push_str("  </Contents>\n");
    }

    for prefix in &result.common_prefixes {
        xml.push_str("  <CommonPrefixes>\n");
        xml.push_str("    <Prefix>");
        xml.push_str(&xml_escape(prefix));
        xml.push_str("</Prefix>\n");
        xml.push_str("  </CommonPrefixes>\n");
    }

    xml.push_str("</ListBucketResult>");
    xml
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ============= Phase 2: Advanced XML Operations =============

use chrono::{DateTime, Utc};
use quick_xml::de::from_str;
use serde::Deserialize;

/// Generate CopyObject response XML
pub fn copy_object_response(etag: &str, last_modified: &DateTime<Utc>) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<CopyObjectResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <LastModified>{}</LastModified>
  <ETag>"{}"</ETag>
</CopyObjectResult>"#,
        format_s3_datetime(last_modified),
        etag
    )
}

// ============= Delete Objects =============

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteObjectsRequest {
    pub quiet: Option<bool>,
    #[serde(rename = "Object", default)]
    pub objects: Vec<ObjectIdentifier>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectIdentifier {
    pub key: String,
    pub version_id: Option<String>,
}

pub fn parse_delete_objects(body: &[u8]) -> Result<DeleteObjectsRequest, quick_xml::DeError> {
    let xml_str = String::from_utf8_lossy(body);
    from_str(&xml_str)
}

#[derive(Debug)]
pub struct DeletedObject {
    pub key: String,
    pub version_id: Option<String>,
    pub delete_marker: bool,
    pub delete_marker_version_id: Option<String>,
}

#[derive(Debug)]
pub struct DeleteError {
    pub key: String,
    pub version_id: Option<String>,
    pub code: String,
    pub message: String,
}

pub fn delete_objects_response(deleted: &[DeletedObject], errors: &[DeleteError]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<DeleteResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#,
    );

    for d in deleted {
        xml.push_str("\n  <Deleted>\n    <Key>");
        xml.push_str(&xml_escape(&d.key));
        xml.push_str("</Key>");
        if let Some(ref vid) = d.version_id {
            xml.push_str("\n    <VersionId>");
            xml.push_str(vid);
            xml.push_str("</VersionId>");
        }
        if d.delete_marker {
            xml.push_str("\n    <DeleteMarker>true</DeleteMarker>");
        }
        xml.push_str("\n  </Deleted>");
    }

    for e in errors {
        xml.push_str("\n  <Error>\n    <Key>");
        xml.push_str(&xml_escape(&e.key));
        xml.push_str("</Key>");
        if let Some(ref vid) = e.version_id {
            xml.push_str("\n    <VersionId>");
            xml.push_str(vid);
            xml.push_str("</VersionId>");
        }
        xml.push_str("\n    <Code>");
        xml.push_str(&e.code);
        xml.push_str("</Code>\n    <Message>");
        xml.push_str(&xml_escape(&e.message));
        xml.push_str("</Message>\n  </Error>");
    }

    xml.push_str("\n</DeleteResult>");
    xml
}

// ============= Multipart Upload =============

pub fn initiate_multipart_upload_response(bucket: &str, key: &str, upload_id: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Bucket>{}</Bucket>
  <Key>{}</Key>
  <UploadId>{}</UploadId>
</InitiateMultipartUploadResult>"#,
        xml_escape(bucket),
        xml_escape(key),
        upload_id
    )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CompleteMultipartUploadRequest {
    #[serde(rename = "Part", default)]
    pub parts: Vec<CompletedPart>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CompletedPart {
    pub part_number: i32,
    #[serde(rename = "ETag")]
    pub etag: String,
}

pub fn parse_complete_multipart(
    body: &[u8],
) -> Result<CompleteMultipartUploadRequest, quick_xml::DeError> {
    let xml_str = String::from_utf8_lossy(body);
    from_str(&xml_str)
}

pub fn complete_multipart_upload_response(bucket: &str, key: &str, etag: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<CompleteMultipartUploadResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Location>/{}/{}</Location>
  <Bucket>{}</Bucket>
  <Key>{}</Key>
  <ETag>"{}"</ETag>
</CompleteMultipartUploadResult>"#,
        xml_escape(bucket),
        xml_escape(key),
        xml_escape(bucket),
        xml_escape(key),
        etag
    )
}

/// Part info for list parts response
pub struct PartInfo {
    pub part_number: i32,
    pub last_modified: DateTime<Utc>,
    pub etag: String,
    pub size: i64,
}

pub fn list_parts_response(
    bucket: &str,
    key: &str,
    upload_id: &str,
    initiator_id: &str,
    storage_class: &str,
    parts: &[PartInfo],
    max_parts: i32,
    is_truncated: bool,
    next_part_number_marker: Option<i32>,
) -> String {
    let mut xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ListPartsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Bucket>{}</Bucket>
  <Key>{}</Key>
  <UploadId>{}</UploadId>
  <Initiator>
    <ID>{}</ID>
    <DisplayName>{}</DisplayName>
  </Initiator>
  <Owner>
    <ID>{}</ID>
    <DisplayName>{}</DisplayName>
  </Owner>
  <StorageClass>{}</StorageClass>
  <PartNumberMarker>0</PartNumberMarker>
  <MaxParts>{}</MaxParts>
  <IsTruncated>{}</IsTruncated>"#,
        xml_escape(bucket),
        xml_escape(key),
        upload_id,
        initiator_id,
        initiator_id,
        initiator_id,
        initiator_id,
        storage_class,
        max_parts,
        is_truncated
    );

    if let Some(marker) = next_part_number_marker {
        xml.push_str(&format!(
            "\n  <NextPartNumberMarker>{}</NextPartNumberMarker>",
            marker
        ));
    }

    for part in parts {
        xml.push_str(&format!(
            r#"
  <Part>
    <PartNumber>{}</PartNumber>
    <LastModified>{}</LastModified>
    <ETag>"{}"</ETag>
    <Size>{}</Size>
  </Part>"#,
            part.part_number,
            format_s3_datetime(&part.last_modified),
            part.etag,
            part.size
        ));
    }

    xml.push_str("\n</ListPartsResult>");
    xml
}

/// Upload info for list multipart uploads response
pub struct UploadInfo {
    pub key: String,
    pub upload_id: String,
    pub initiator_id: String,
    pub storage_class: String,
    pub initiated: DateTime<Utc>,
}

pub fn list_multipart_uploads_response(
    bucket: &str,
    prefix: Option<&str>,
    delimiter: Option<&str>,
    key_marker: Option<&str>,
    upload_id_marker: Option<&str>,
    max_uploads: i32,
    is_truncated: bool,
    uploads: &[UploadInfo],
) -> String {
    let mut xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ListMultipartUploadsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Bucket>{}</Bucket>"#,
        xml_escape(bucket)
    );

    if let Some(p) = prefix {
        xml.push_str(&format!("\n  <Prefix>{}</Prefix>", xml_escape(p)));
    } else {
        xml.push_str("\n  <Prefix></Prefix>");
    }

    if let Some(d) = delimiter {
        xml.push_str(&format!("\n  <Delimiter>{}</Delimiter>", xml_escape(d)));
    }

    if let Some(km) = key_marker {
        xml.push_str(&format!("\n  <KeyMarker>{}</KeyMarker>", xml_escape(km)));
    } else {
        xml.push_str("\n  <KeyMarker></KeyMarker>");
    }

    if let Some(um) = upload_id_marker {
        xml.push_str(&format!("\n  <UploadIdMarker>{}</UploadIdMarker>", um));
    } else {
        xml.push_str("\n  <UploadIdMarker></UploadIdMarker>");
    }

    xml.push_str(&format!(
        r#"
  <MaxUploads>{}</MaxUploads>
  <IsTruncated>{}</IsTruncated>"#,
        max_uploads, is_truncated
    ));

    for upload in uploads {
        xml.push_str(&format!(
            r#"
  <Upload>
    <Key>{}</Key>
    <UploadId>{}</UploadId>
    <Initiator>
      <ID>{}</ID>
      <DisplayName>{}</DisplayName>
    </Initiator>
    <Owner>
      <ID>{}</ID>
      <DisplayName>{}</DisplayName>
    </Owner>
    <StorageClass>{}</StorageClass>
    <Initiated>{}</Initiated>
  </Upload>"#,
            xml_escape(&upload.key),
            upload.upload_id,
            upload.initiator_id,
            upload.initiator_id,
            upload.initiator_id,
            upload.initiator_id,
            upload.storage_class,
            format_s3_datetime(&upload.initiated)
        ));
    }

    xml.push_str("\n</ListMultipartUploadsResult>");
    xml
}

// ============= Bucket Versioning =============

use hafiz_core::types::{DeleteMarker, ObjectVersion, VersioningStatus};

/// Generate GetBucketVersioning response XML
pub fn get_bucket_versioning_response(status: &VersioningStatus) -> String {
    let status_str = status.as_str();
    if status_str.is_empty() {
        // Unversioned bucket returns empty versioning config
        r#"<?xml version="1.0" encoding="UTF-8"?>
<VersioningConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/"/>"#
            .to_string()
    } else {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<VersioningConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Status>{}</Status>
</VersioningConfiguration>"#,
            status_str
        )
    }
}

/// Parse PutBucketVersioning request XML
pub fn parse_versioning_configuration(body: &[u8]) -> Result<VersioningStatus, quick_xml::DeError> {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct VersioningConfiguration {
        status: Option<String>,
    }

    let xml_str = String::from_utf8_lossy(body);
    let config: VersioningConfiguration = from_str(&xml_str)?;

    Ok(match config.status.as_deref() {
        Some("Enabled") => VersioningStatus::Enabled,
        Some("Suspended") => VersioningStatus::Suspended,
        _ => VersioningStatus::Unversioned,
    })
}

/// Generate ListObjectVersions response XML
pub fn list_object_versions_response(
    bucket: &str,
    prefix: Option<&str>,
    delimiter: Option<&str>,
    key_marker: Option<&str>,
    version_id_marker: Option<&str>,
    max_keys: i32,
    is_truncated: bool,
    versions: &[ObjectVersion],
    delete_markers: &[DeleteMarker],
    common_prefixes: &[String],
    next_key_marker: Option<&str>,
    next_version_id_marker: Option<&str>,
) -> String {
    let mut xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ListVersionsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Name>{}</Name>"#,
        xml_escape(bucket)
    );

    if let Some(p) = prefix {
        xml.push_str(&format!("\n  <Prefix>{}</Prefix>", xml_escape(p)));
    } else {
        xml.push_str("\n  <Prefix></Prefix>");
    }

    if let Some(km) = key_marker {
        xml.push_str(&format!("\n  <KeyMarker>{}</KeyMarker>", xml_escape(km)));
    } else {
        xml.push_str("\n  <KeyMarker></KeyMarker>");
    }

    if let Some(vim) = version_id_marker {
        xml.push_str(&format!("\n  <VersionIdMarker>{}</VersionIdMarker>", vim));
    } else {
        xml.push_str("\n  <VersionIdMarker></VersionIdMarker>");
    }

    xml.push_str(&format!(
        r#"
  <MaxKeys>{}</MaxKeys>
  <IsTruncated>{}</IsTruncated>"#,
        max_keys, is_truncated
    ));

    if let Some(nkm) = next_key_marker {
        xml.push_str(&format!(
            "\n  <NextKeyMarker>{}</NextKeyMarker>",
            xml_escape(nkm)
        ));
    }

    if let Some(nvim) = next_version_id_marker {
        xml.push_str(&format!(
            "\n  <NextVersionIdMarker>{}</NextVersionIdMarker>",
            nvim
        ));
    }

    if let Some(d) = delimiter {
        xml.push_str(&format!("\n  <Delimiter>{}</Delimiter>", xml_escape(d)));
    }

    // Add versions
    for v in versions {
        xml.push_str(&format!(
            r#"
  <Version>
    <Key>{}</Key>
    <VersionId>{}</VersionId>
    <IsLatest>{}</IsLatest>
    <LastModified>{}</LastModified>
    <ETag>"{}"</ETag>
    <Size>{}</Size>
    <Owner>
      <ID>{}</ID>
      <DisplayName>{}</DisplayName>
    </Owner>
    <StorageClass>{}</StorageClass>
  </Version>"#,
            xml_escape(&v.key),
            v.version_id,
            v.is_latest,
            format_s3_datetime(&v.last_modified),
            v.etag,
            v.size,
            v.owner_id,
            v.owner_id,
            v.storage_class
        ));
    }

    // Add delete markers
    for dm in delete_markers {
        xml.push_str(&format!(
            r#"
  <DeleteMarker>
    <Key>{}</Key>
    <VersionId>{}</VersionId>
    <IsLatest>{}</IsLatest>
    <LastModified>{}</LastModified>
    <Owner>
      <ID>{}</ID>
      <DisplayName>{}</DisplayName>
    </Owner>
  </DeleteMarker>"#,
            xml_escape(&dm.key),
            dm.version_id,
            dm.is_latest,
            format_s3_datetime(&dm.last_modified),
            dm.owner_id,
            dm.owner_id
        ));
    }

    // Add common prefixes
    for cp in common_prefixes {
        xml.push_str(&format!(
            r#"
  <CommonPrefixes>
    <Prefix>{}</Prefix>
  </CommonPrefixes>"#,
            xml_escape(cp)
        ));
    }

    xml.push_str("\n</ListVersionsResult>");
    xml
}

// ============= Object Tagging =============

use hafiz_core::types::{
    Expiration, LifecycleConfiguration, LifecycleFilter, LifecycleRule, RuleStatus, Tag, TagSet,
};

/// Generate GetObjectTagging response XML
pub fn get_object_tagging_response(tags: &TagSet) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Tagging xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <TagSet>"#,
    );

    for tag in &tags.tags {
        xml.push_str(&format!(
            r#"
    <Tag>
      <Key>{}</Key>
      <Value>{}</Value>
    </Tag>"#,
            xml_escape(&tag.key),
            xml_escape(&tag.value)
        ));
    }

    xml.push_str(
        r#"
  </TagSet>
</Tagging>"#,
    );
    xml
}

/// Parse PutObjectTagging request XML
pub fn parse_tagging(body: &[u8]) -> Result<TagSet, quick_xml::DeError> {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Tagging {
        tag_set: TagSetXml,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct TagSetXml {
        #[serde(rename = "Tag", default)]
        tags: Vec<TagXml>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct TagXml {
        key: String,
        value: String,
    }

    let xml_str = String::from_utf8_lossy(body);
    let tagging: Tagging = from_str(&xml_str)?;

    let mut tag_set = TagSet::new();
    for t in tagging.tag_set.tags {
        tag_set.tags.push(Tag::new(t.key, t.value));
    }

    Ok(tag_set)
}

// ============= Bucket Lifecycle =============

/// Generate GetBucketLifecycle response XML
pub fn get_bucket_lifecycle_response(config: &LifecycleConfiguration) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<LifecycleConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#,
    );

    for rule in &config.rules {
        xml.push_str("\n  <Rule>");
        xml.push_str(&format!("\n    <ID>{}</ID>", xml_escape(&rule.id)));

        // Filter
        match &rule.filter {
            LifecycleFilter::All => {
                xml.push_str("\n    <Filter></Filter>");
            }
            LifecycleFilter::Prefix(p) => {
                xml.push_str(&format!(
                    "\n    <Filter>\n      <Prefix>{}</Prefix>\n    </Filter>",
                    xml_escape(p)
                ));
            }
            LifecycleFilter::Tag(t) => {
                xml.push_str(&format!(
                    r#"
    <Filter>
      <Tag>
        <Key>{}</Key>
        <Value>{}</Value>
      </Tag>
    </Filter>"#,
                    xml_escape(&t.key),
                    xml_escape(&t.value)
                ));
            }
            LifecycleFilter::And { prefix, tags } => {
                xml.push_str("\n    <Filter>\n      <And>");
                if let Some(p) = prefix {
                    xml.push_str(&format!("\n        <Prefix>{}</Prefix>", xml_escape(p)));
                }
                for t in tags {
                    xml.push_str(&format!(
                        r#"
        <Tag>
          <Key>{}</Key>
          <Value>{}</Value>
        </Tag>"#,
                        xml_escape(&t.key),
                        xml_escape(&t.value)
                    ));
                }
                xml.push_str("\n      </And>\n    </Filter>");
            }
        }

        // Status
        xml.push_str(&format!(
            "\n    <Status>{}</Status>",
            if rule.status == RuleStatus::Enabled {
                "Enabled"
            } else {
                "Disabled"
            }
        ));

        // Expiration
        if let Some(ref exp) = rule.expiration {
            match exp {
                Expiration::Days(d) => {
                    xml.push_str(&format!(
                        "\n    <Expiration>\n      <Days>{}</Days>\n    </Expiration>",
                        d
                    ));
                }
                Expiration::Date(date) => {
                    xml.push_str(&format!(
                        "\n    <Expiration>\n      <Date>{}</Date>\n    </Expiration>",
                        date
                    ));
                }
                Expiration::ExpiredObjectDeleteMarker => {
                    xml.push_str("\n    <Expiration>\n      <ExpiredObjectDeleteMarker>true</ExpiredObjectDeleteMarker>\n    </Expiration>");
                }
            }
        }

        // NoncurrentVersionExpiration
        if let Some(ref nve) = rule.noncurrent_version_expiration {
            xml.push_str(&format!(
                "\n    <NoncurrentVersionExpiration>\n      <NoncurrentDays>{}</NoncurrentDays>\n    </NoncurrentVersionExpiration>",
                nve.noncurrent_days
            ));
        }

        // AbortIncompleteMultipartUpload
        if let Some(ref abort) = rule.abort_incomplete_multipart_upload {
            xml.push_str(&format!(
                "\n    <AbortIncompleteMultipartUpload>\n      <DaysAfterInitiation>{}</DaysAfterInitiation>\n    </AbortIncompleteMultipartUpload>",
                abort.days_after_initiation
            ));
        }

        xml.push_str("\n  </Rule>");
    }

    xml.push_str("\n</LifecycleConfiguration>");
    xml
}

/// Parse PutBucketLifecycle request XML
pub fn parse_lifecycle_configuration(
    body: &[u8],
) -> Result<LifecycleConfiguration, quick_xml::DeError> {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct LifecycleConfigurationXml {
        #[serde(rename = "Rule", default)]
        rules: Vec<RuleXml>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct RuleXml {
        #[serde(rename = "ID")]
        id: String,
        filter: Option<FilterXml>,
        status: String,
        expiration: Option<ExpirationXml>,
        noncurrent_version_expiration: Option<NoncurrentVersionExpirationXml>,
        abort_incomplete_multipart_upload: Option<AbortIncompleteMultipartUploadXml>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct FilterXml {
        prefix: Option<String>,
        tag: Option<TagXmlSimple>,
        and: Option<AndXml>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct AndXml {
        prefix: Option<String>,
        #[serde(rename = "Tag", default)]
        tags: Vec<TagXmlSimple>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct TagXmlSimple {
        key: String,
        value: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct ExpirationXml {
        days: Option<u32>,
        date: Option<String>,
        expired_object_delete_marker: Option<bool>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct NoncurrentVersionExpirationXml {
        noncurrent_days: u32,
        newer_noncurrent_versions: Option<u32>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct AbortIncompleteMultipartUploadXml {
        days_after_initiation: u32,
    }

    let xml_str = String::from_utf8_lossy(body);
    let config: LifecycleConfigurationXml = from_str(&xml_str)?;

    let mut lifecycle = LifecycleConfiguration::new();

    for r in config.rules {
        let mut rule = LifecycleRule::new(&r.id);

        // Parse status
        rule.status = if r.status.to_lowercase() == "enabled" {
            RuleStatus::Enabled
        } else {
            RuleStatus::Disabled
        };

        // Parse filter
        if let Some(f) = r.filter {
            if let Some(and) = f.and {
                let tags: Vec<Tag> = and
                    .tags
                    .into_iter()
                    .map(|t| Tag::new(t.key, t.value))
                    .collect();
                rule.filter = LifecycleFilter::And {
                    prefix: and.prefix,
                    tags,
                };
            } else if let Some(tag) = f.tag {
                rule.filter = LifecycleFilter::Tag(Tag::new(tag.key, tag.value));
            } else if let Some(prefix) = f.prefix {
                rule.filter = LifecycleFilter::Prefix(prefix);
            } else {
                rule.filter = LifecycleFilter::All;
            }
        }

        // Parse expiration
        if let Some(exp) = r.expiration {
            if let Some(days) = exp.days {
                rule.expiration = Some(Expiration::Days(days));
            } else if let Some(date_str) = exp.date {
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                    rule.expiration = Some(Expiration::Date(date));
                }
            } else if exp.expired_object_delete_marker == Some(true) {
                rule.expiration = Some(Expiration::ExpiredObjectDeleteMarker);
            }
        }

        // Parse noncurrent version expiration
        if let Some(nve) = r.noncurrent_version_expiration {
            rule.noncurrent_version_expiration =
                Some(hafiz_core::types::NoncurrentVersionExpiration {
                    noncurrent_days: nve.noncurrent_days,
                    newer_noncurrent_versions: nve.newer_noncurrent_versions,
                });
        }

        // Parse abort incomplete multipart upload
        if let Some(abort) = r.abort_incomplete_multipart_upload {
            rule.abort_incomplete_multipart_upload =
                Some(hafiz_core::types::AbortIncompleteMultipartUpload {
                    days_after_initiation: abort.days_after_initiation,
                });
        }

        lifecycle.rules.push(rule);
    }

    Ok(lifecycle)
}
