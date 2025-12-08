//! head command - get object metadata

use super::CommandContext;
use crate::s3_client::{create_client, S3Uri};
use crate::utils::format_size;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::Serialize;

#[derive(Serialize)]
struct HeadResult {
    bucket: String,
    key: String,
    content_type: Option<String>,
    content_length: Option<i64>,
    last_modified: Option<String>,
    etag: Option<String>,
    storage_class: Option<String>,
    version_id: Option<String>,
    metadata: std::collections::HashMap<String, String>,
}

pub async fn execute(ctx: &CommandContext, path: &str) -> Result<()> {
    let client = create_client(&ctx.config).await?;
    let uri = S3Uri::parse(path)?;
    let key = uri.key.as_ref().context("Object key required")?;

    ctx.debug(&format!("Getting metadata for s3://{}/{}", uri.bucket, key));

    let resp = client
        .head_object()
        .bucket(&uri.bucket)
        .key(key)
        .send()
        .await
        .context("Failed to get object metadata")?;

    let last_modified = resp.last_modified().map(|d| {
        DateTime::<Utc>::from_timestamp(d.secs(), 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_default()
    });

    let metadata: std::collections::HashMap<String, String> = resp
        .metadata()
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();

    if ctx.is_json() {
        let result = HeadResult {
            bucket: uri.bucket.clone(),
            key: key.clone(),
            content_type: resp.content_type().map(|s| s.to_string()),
            content_length: resp.content_length(),
            last_modified,
            etag: resp.e_tag().map(|s| s.to_string()),
            storage_class: resp.storage_class().map(|s| s.as_str().to_string()),
            version_id: resp.version_id().map(|s| s.to_string()),
            metadata,
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{}", format!("s3://{}/{}", uri.bucket, key).blue().bold());
        println!();

        if let Some(ct) = resp.content_type() {
            println!("  {}: {}", "Content-Type".cyan(), ct);
        }
        if let Some(len) = resp.content_length() {
            println!(
                "  {}: {} ({})",
                "Content-Length".cyan(),
                len,
                format_size(len, true)
            );
        }
        if let Some(lm) = &last_modified {
            println!("  {}: {}", "Last-Modified".cyan(), lm);
        }
        if let Some(etag) = resp.e_tag() {
            println!("  {}: {}", "ETag".cyan(), etag);
        }
        if let Some(sc) = resp.storage_class() {
            println!("  {}: {}", "Storage-Class".cyan(), sc.as_str());
        }
        if let Some(vid) = resp.version_id() {
            println!("  {}: {}", "Version-Id".cyan(), vid);
        }

        if !metadata.is_empty() {
            println!();
            println!("  {}:", "Metadata".cyan());
            for (k, v) in &metadata {
                println!("    {}: {}", k, v);
            }
        }
    }

    Ok(())
}
