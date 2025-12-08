//! cat command - stream object content to stdout

use super::CommandContext;
use crate::s3_client::{create_client, S3Uri};
use anyhow::{Context, Result};
use tokio::io::{stdout, AsyncReadExt, AsyncWriteExt};

pub async fn execute(ctx: &CommandContext, path: &str) -> Result<()> {
    let client = create_client(&ctx.config).await?;
    let uri = S3Uri::parse(path)?;
    let key = uri.key.as_ref().context("Object key required")?;

    ctx.debug(&format!("Streaming s3://{}/{}", uri.bucket, key));

    let resp = client
        .get_object()
        .bucket(&uri.bucket)
        .key(key)
        .send()
        .await
        .context("Failed to get object")?;

    let mut stream = resp.body.into_async_read();
    let mut stdout = stdout();

    let mut buf = [0u8; 8192];
    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        stdout.write_all(&buf[..n]).await?;
    }

    stdout.flush().await?;

    Ok(())
}
