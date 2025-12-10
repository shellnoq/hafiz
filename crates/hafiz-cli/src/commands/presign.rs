//! presign command - generate presigned URLs

use super::CommandContext;
use crate::s3_client::{create_client, S3Uri};
use anyhow::{Context, Result};
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;

pub async fn execute(ctx: &CommandContext, path: &str, expires: u64, method: &str) -> Result<()> {
    let client = create_client(&ctx.config).await?;
    let uri = S3Uri::parse(path)?;
    let key = uri.key.as_ref().context("Object key required")?;

    ctx.debug(&format!(
        "Generating presigned URL for s3://{}/{} ({} method, {} seconds)",
        uri.bucket, key, method, expires
    ));

    let presign_config = PresigningConfig::builder()
        .expires_in(Duration::from_secs(expires))
        .build()?;

    let url = match method.to_uppercase().as_str() {
        "GET" => {
            let req = client
                .get_object()
                .bucket(&uri.bucket)
                .key(key)
                .presigned(presign_config)
                .await?;
            req.uri().to_string()
        }
        "PUT" => {
            let req = client
                .put_object()
                .bucket(&uri.bucket)
                .key(key)
                .presigned(presign_config)
                .await?;
            req.uri().to_string()
        }
        _ => anyhow::bail!("Unsupported method: {}. Use GET or PUT.", method),
    };

    println!("{}", url);

    Ok(())
}
