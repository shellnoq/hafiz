//! rb command - remove bucket

use super::rm::{execute as rm_execute, RmOptions};
use super::CommandContext;
use crate::s3_client::{create_client, S3Uri};
use crate::utils::confirm;
use anyhow::{Context, Result};
use colored::Colorize;

pub async fn execute(ctx: &CommandContext, bucket: &str, force: bool) -> Result<()> {
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

    // If force, delete all objects first
    if force {
        if !ctx.quiet {
            let msg = format!(
                "Delete all objects in bucket '{}' and remove bucket?",
                bucket_name
            );
            if !confirm(&msg) {
                ctx.info("Cancelled");
                return Ok(());
            }
        }

        ctx.debug(&format!("Deleting all objects in bucket: {}", bucket_name));

        let rm_opts = RmOptions {
            recursive: true,
            force: true,
            include: None,
            exclude: None,
            dryrun: false,
        };

        let s3_path = format!("s3://{}/", bucket_name);
        rm_execute(ctx, &s3_path, rm_opts).await?;
    }

    ctx.debug(&format!("Removing bucket: {}", bucket_name));

    client
        .delete_bucket()
        .bucket(&bucket_name)
        .send()
        .await
        .context("Failed to delete bucket. Bucket may not be empty (use --force).")?;

    if !ctx.quiet {
        println!("{}: s3://{}", "remove_bucket".red(), bucket_name);
    }

    Ok(())
}
