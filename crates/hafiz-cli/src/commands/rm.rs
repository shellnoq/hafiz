//! rm command - remove objects

use super::CommandContext;
use crate::s3_client::{create_client, S3Uri};
use crate::utils::{confirm, matches_patterns};
use anyhow::{Context, Result};
use aws_sdk_s3::types::{Delete, ObjectIdentifier};
use colored::Colorize;

pub struct RmOptions {
    pub recursive: bool,
    pub force: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub dryrun: bool,
}

pub async fn execute(ctx: &CommandContext, path: &str, opts: RmOptions) -> Result<()> {
    let client = create_client(&ctx.config).await?;
    let uri = S3Uri::parse(path)?;

    if uri.key.is_none() && !opts.recursive {
        anyhow::bail!("Cannot delete bucket contents without --recursive flag");
    }

    if uri.is_prefix() || opts.recursive {
        // Delete multiple objects
        delete_prefix(ctx, &client, &uri, &opts).await
    } else {
        // Delete single object
        delete_object(ctx, &client, &uri, &opts).await
    }
}

async fn delete_object(
    ctx: &CommandContext,
    client: &aws_sdk_s3::Client,
    uri: &S3Uri,
    opts: &RmOptions,
) -> Result<()> {
    let key = uri.key.as_ref().context("Object key required")?;

    if !opts.force && !ctx.quiet {
        let msg = format!("Delete s3://{}/{}?", uri.bucket, key);
        if !confirm(&msg) {
            ctx.info("Cancelled");
            return Ok(());
        }
    }

    if opts.dryrun {
        println!("(dryrun) delete: s3://{}/{}", uri.bucket, key);
        return Ok(());
    }

    ctx.debug(&format!("Deleting s3://{}/{}", uri.bucket, key));

    client
        .delete_object()
        .bucket(&uri.bucket)
        .key(key)
        .send()
        .await
        .context("Delete failed")?;

    if !ctx.quiet {
        println!("{}: s3://{}/{}", "delete".red(), uri.bucket, key);
    }

    Ok(())
}

async fn delete_prefix(
    ctx: &CommandContext,
    client: &aws_sdk_s3::Client,
    uri: &S3Uri,
    opts: &RmOptions,
) -> Result<()> {
    let prefix = uri.key.clone().unwrap_or_default();

    // List all objects
    let mut objects: Vec<String> = Vec::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let mut req = client
            .list_objects_v2()
            .bucket(&uri.bucket)
            .prefix(&prefix);

        if let Some(token) = &continuation_token {
            req = req.continuation_token(token);
        }

        let resp = req.send().await?;

        if let Some(contents) = resp.contents {
            for obj in contents {
                if let Some(key) = obj.key() {
                    // Check patterns
                    if matches_patterns(key, opts.include.as_deref(), opts.exclude.as_deref())? {
                        objects.push(key.to_string());
                    }
                }
            }
        }

        if resp.is_truncated.unwrap_or(false) {
            continuation_token = resp.next_continuation_token;
        } else {
            break;
        }
    }

    if objects.is_empty() {
        ctx.info("No objects to delete");
        return Ok(());
    }

    if !opts.force && !ctx.quiet {
        let msg = format!("Delete {} object(s) from s3://{}?", objects.len(), uri.bucket);
        if !confirm(&msg) {
            ctx.info("Cancelled");
            return Ok(());
        }
    }

    // Delete in batches of 1000 (S3 limit)
    let total = objects.len();
    let mut deleted = 0;

    for chunk in objects.chunks(1000) {
        if opts.dryrun {
            for key in chunk {
                println!("(dryrun) delete: s3://{}/{}", uri.bucket, key);
            }
            deleted += chunk.len();
            continue;
        }

        let delete_objects: Vec<ObjectIdentifier> = chunk
            .iter()
            .map(|key| ObjectIdentifier::builder().key(key).build().unwrap())
            .collect();

        let delete = Delete::builder()
            .set_objects(Some(delete_objects))
            .build()?;

        let resp = client
            .delete_objects()
            .bucket(&uri.bucket)
            .delete(delete)
            .send()
            .await?;

        if let Some(deleted_objs) = resp.deleted {
            for obj in deleted_objs {
                if !ctx.quiet {
                    println!(
                        "{}: s3://{}/{}",
                        "delete".red(),
                        uri.bucket,
                        obj.key().unwrap_or("")
                    );
                }
            }
            deleted += deleted_objs.len();
        }

        if let Some(errors) = resp.errors {
            for err in errors {
                ctx.error(&format!(
                    "Failed to delete {}: {}",
                    err.key().unwrap_or(""),
                    err.message().unwrap_or("")
                ));
            }
        }
    }

    if !ctx.quiet {
        println!("\nDeleted {} of {} object(s)", deleted, total);
    }

    Ok(())
}
