//! mb command - make bucket

use super::CommandContext;
use crate::s3_client::{create_client, S3Uri};
use anyhow::{Context, Result};
use aws_sdk_s3::types::{BucketLocationConstraint, CreateBucketConfiguration};
use colored::Colorize;

pub async fn execute(
    ctx: &CommandContext,
    bucket: &str,
    region: Option<String>,
) -> Result<()> {
    let client = create_client(&ctx.config).await?;

    // Parse bucket name from s3:// URI if provided
    let bucket_name = if bucket.starts_with("s3://") {
        let uri = S3Uri::parse(bucket)?;
        uri.bucket
    } else {
        bucket.to_string()
    };

    if bucket_name.is_empty() {
        anyhow::bail!("Bucket name cannot be empty");
    }

    ctx.debug(&format!("Creating bucket: {}", bucket_name));

    let mut req = client.create_bucket().bucket(&bucket_name);

    // Add region constraint if not us-east-1
    let region_str = region.as_deref().unwrap_or(&ctx.config.region);
    if region_str != "us-east-1" {
        let constraint = BucketLocationConstraint::from(region_str);
        let config = CreateBucketConfiguration::builder()
            .location_constraint(constraint)
            .build();
        req = req.create_bucket_configuration(config);
    }

    req.send().await.context("Failed to create bucket")?;

    if !ctx.quiet {
        println!(
            "{}: s3://{}",
            "make_bucket".green(),
            bucket_name
        );
    }

    Ok(())
}
