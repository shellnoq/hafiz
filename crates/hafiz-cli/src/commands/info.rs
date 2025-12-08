//! info command - display bucket or object information

use super::CommandContext;
use crate::s3_client::{create_client, S3Uri};
use crate::utils::format_size;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::Serialize;

#[derive(Serialize)]
struct BucketInfoResult {
    name: String,
    region: Option<String>,
    versioning: Option<String>,
    object_count: usize,
    total_size: i64,
}

pub async fn execute(ctx: &CommandContext, path: &str) -> Result<()> {
    let client = create_client(&ctx.config).await?;
    let uri = S3Uri::parse(path)?;

    if uri.key.is_none() || uri.key.as_deref() == Some("") {
        // Bucket info
        bucket_info(ctx, &client, &uri.bucket).await
    } else {
        // Object info (same as head)
        super::head::execute(ctx, path).await
    }
}

async fn bucket_info(
    ctx: &CommandContext,
    client: &aws_sdk_s3::Client,
    bucket: &str,
) -> Result<()> {
    ctx.debug(&format!("Getting info for bucket: {}", bucket));

    // Get bucket location
    let location = client
        .get_bucket_location()
        .bucket(bucket)
        .send()
        .await
        .ok()
        .and_then(|r| r.location_constraint().map(|l| l.as_str().to_string()));

    // Get versioning status
    let versioning = client
        .get_bucket_versioning()
        .bucket(bucket)
        .send()
        .await
        .ok()
        .and_then(|r| r.status().map(|s| s.as_str().to_string()));

    // Count objects and size
    let mut object_count = 0;
    let mut total_size: i64 = 0;
    let mut continuation_token: Option<String> = None;

    loop {
        let mut req = client.list_objects_v2().bucket(bucket);

        if let Some(token) = &continuation_token {
            req = req.continuation_token(token);
        }

        let resp = req.send().await?;

        if let Some(contents) = resp.contents {
            for obj in contents {
                object_count += 1;
                if let Some(size) = obj.size() {
                    total_size += size;
                }
            }
        }

        if resp.is_truncated.unwrap_or(false) {
            continuation_token = resp.next_continuation_token;
        } else {
            break;
        }
    }

    if ctx.is_json() {
        let result = BucketInfoResult {
            name: bucket.to_string(),
            region: location,
            versioning,
            object_count,
            total_size,
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{}", format!("s3://{}", bucket).blue().bold());
        println!();

        if let Some(loc) = &location {
            println!("  {}: {}", "Region".cyan(), loc);
        } else {
            println!("  {}: us-east-1 (default)", "Region".cyan());
        }

        if let Some(ver) = &versioning {
            println!("  {}: {}", "Versioning".cyan(), ver);
        } else {
            println!("  {}: Disabled", "Versioning".cyan());
        }

        println!("  {}: {}", "Objects".cyan(), object_count);
        println!(
            "  {}: {} ({})",
            "Total Size".cyan(),
            total_size,
            format_size(total_size, true)
        );
    }

    Ok(())
}
