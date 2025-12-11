//! sync command - synchronize directories

use super::CommandContext;
use crate::progress::{create_spinner, format_bytes};
use crate::s3_client::{create_client, is_s3_uri, S3Uri, TransferDirection};
use crate::utils::{guess_content_type, matches_patterns};
use anyhow::{Context, Result};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{Delete, ObjectIdentifier};
use chrono::{DateTime, Utc};
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;

pub struct SyncOptions {
    pub delete: bool,
    pub exclude: Option<String>,
    pub include: Option<String>,
    pub size_only: bool,
    pub dryrun: bool,
    pub parallel: usize,
}

#[derive(Debug)]
struct FileInfo {
    size: i64,
    last_modified: Option<DateTime<Utc>>,
}

pub async fn execute(
    ctx: &CommandContext,
    source: &str,
    destination: &str,
    opts: SyncOptions,
) -> Result<()> {
    let direction = TransferDirection::determine(source, destination);

    match direction {
        TransferDirection::Upload => sync_upload(ctx, source, destination, &opts).await,
        TransferDirection::Download => sync_download(ctx, source, destination, &opts).await,
        TransferDirection::S3ToS3 => {
            anyhow::bail!("S3 to S3 sync is not yet supported. Use cp --recursive instead.")
        }
        TransferDirection::LocalToLocal => {
            anyhow::bail!("Local to local sync is not supported. Use rsync instead.")
        }
    }
}

async fn sync_upload(
    ctx: &CommandContext,
    source: &str,
    destination: &str,
    opts: &SyncOptions,
) -> Result<()> {
    let client = create_client(&ctx.config).await?;
    let dest_uri = S3Uri::parse(destination)?;
    let source_path = Path::new(source);

    if !source_path.is_dir() {
        anyhow::bail!("Source must be a directory for sync");
    }

    let spinner = if !ctx.quiet {
        Some(create_spinner("Scanning..."))
    } else {
        None
    };

    // Get local files
    let mut local_files: HashMap<String, FileInfo> = HashMap::new();
    for entry in WalkDir::new(source_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            let relative = path
                .strip_prefix(source_path)
                .unwrap_or(path)
                .to_str()
                .unwrap_or("")
                .replace('\\', "/");

            if !matches_patterns(&relative, opts.include.as_deref(), opts.exclude.as_deref())? {
                continue;
            }

            if let Ok(metadata) = fs::metadata(path).await {
                local_files.insert(
                    relative,
                    FileInfo {
                        size: metadata.len() as i64,
                        last_modified: metadata
                            .modified()
                            .ok()
                            .map(|t| DateTime::<Utc>::from(t)),
                    },
                );
            }
        }
    }

    // Get remote objects
    let prefix = dest_uri.key.clone().unwrap_or_default();
    let mut remote_files: HashMap<String, FileInfo> = HashMap::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let mut req = client
            .list_objects_v2()
            .bucket(&dest_uri.bucket)
            .prefix(&prefix);

        if let Some(token) = &continuation_token {
            req = req.continuation_token(token);
        }

        let resp = req.send().await?;

        if let Some(contents) = resp.contents {
            for obj in contents {
                if let Some(key) = obj.key() {
                    let relative = key.strip_prefix(&prefix).unwrap_or(key);
                    let relative = relative.trim_start_matches('/');

                    if !matches_patterns(relative, opts.include.as_deref(), opts.exclude.as_deref())? {
                        continue;
                    }

                    remote_files.insert(
                        relative.to_string(),
                        FileInfo {
                            size: obj.size().unwrap_or(0),
                            last_modified: obj.last_modified().map(|d| {
                                DateTime::<Utc>::from_timestamp(d.secs(), 0).unwrap_or_default()
                            }),
                        },
                    );
                }
            }
        }

        if resp.is_truncated.unwrap_or(false) {
            continuation_token = resp.next_continuation_token;
        } else {
            break;
        }
    }

    if let Some(s) = spinner {
        s.finish_with_message(format!(
            "Local: {} files, Remote: {} files",
            local_files.len(),
            remote_files.len()
        ));
    }

    // Determine files to upload
    let mut to_upload: Vec<String> = Vec::new();
    for (relative, local_info) in &local_files {
        let needs_upload = match remote_files.get(relative) {
            None => true,
            Some(remote_info) => {
                if opts.size_only {
                    local_info.size != remote_info.size
                } else {
                    // Compare size and time
                    local_info.size != remote_info.size
                        || local_info.last_modified > remote_info.last_modified
                }
            }
        };

        if needs_upload {
            to_upload.push(relative.clone());
        }
    }

    // Determine files to delete
    let mut to_delete: Vec<String> = Vec::new();
    if opts.delete {
        for relative in remote_files.keys() {
            if !local_files.contains_key(relative) {
                to_delete.push(relative.clone());
            }
        }
    }

    if !ctx.quiet {
        println!(
            "To upload: {}, To delete: {}",
            to_upload.len(),
            to_delete.len()
        );
    }

    // Upload files
    let mut uploaded = 0;
    let mut upload_bytes: u64 = 0;

    for relative in &to_upload {
        let local_path = source_path.join(relative);
        let dest_key = if prefix.is_empty() {
            relative.clone()
        } else if prefix.ends_with('/') {
            format!("{}{}", prefix, relative)
        } else {
            format!("{}/{}", prefix, relative)
        };

        if opts.dryrun {
            println!(
                "(dryrun) upload: {} -> s3://{}/{}",
                local_path.display(),
                dest_uri.bucket,
                dest_key
            );
            uploaded += 1;
            continue;
        }

        let metadata = fs::metadata(&local_path).await?;
        let content_type = guess_content_type(relative);
        let body = ByteStream::from_path(&local_path).await?;

        client
            .put_object()
            .bucket(&dest_uri.bucket)
            .key(&dest_key)
            .content_type(content_type)
            .body(body)
            .send()
            .await?;

        uploaded += 1;
        upload_bytes += metadata.len();

        if !ctx.quiet {
            println!(
                "{}: {} -> s3://{}/{}",
                "upload".green(),
                local_path.display(),
                dest_uri.bucket,
                dest_key
            );
        }
    }

    // Delete remote files
    let mut deleted = 0;

    if opts.delete && !to_delete.is_empty() {
        for chunk in to_delete.chunks(1000) {
            let keys_to_delete: Vec<String> = chunk
                .iter()
                .map(|r| {
                    if prefix.is_empty() {
                        r.clone()
                    } else if prefix.ends_with('/') {
                        format!("{}{}", prefix, r)
                    } else {
                        format!("{}/{}", prefix, r)
                    }
                })
                .collect();

            if opts.dryrun {
                for key in &keys_to_delete {
                    println!("(dryrun) delete: s3://{}/{}", dest_uri.bucket, key);
                }
                deleted += keys_to_delete.len();
                continue;
            }

            let delete_objects: Vec<ObjectIdentifier> = keys_to_delete
                .iter()
                .map(|key| ObjectIdentifier::builder().key(key).build().unwrap())
                .collect();

            let delete = Delete::builder()
                .set_objects(Some(delete_objects))
                .build()?;

            let resp = client
                .delete_objects()
                .bucket(&dest_uri.bucket)
                .delete(delete)
                .send()
                .await?;

            if let Some(deleted_objs) = resp.deleted {
                for obj in &deleted_objs {
                    if !ctx.quiet {
                        println!(
                            "{}: s3://{}/{}",
                            "delete".red(),
                            dest_uri.bucket,
                            obj.key().unwrap_or("")
                        );
                    }
                }
                deleted += deleted_objs.len();
            }
        }
    }

    if !ctx.quiet {
        println!(
            "\nSynced: {} uploaded ({}), {} deleted",
            uploaded,
            format_bytes(upload_bytes),
            deleted
        );
    }

    Ok(())
}

async fn sync_download(
    ctx: &CommandContext,
    source: &str,
    destination: &str,
    opts: &SyncOptions,
) -> Result<()> {
    let client = create_client(&ctx.config).await?;
    let source_uri = S3Uri::parse(source)?;
    let dest_path = Path::new(destination);

    if !dest_path.exists() {
        fs::create_dir_all(dest_path).await?;
    }

    let spinner = if !ctx.quiet {
        Some(create_spinner("Scanning..."))
    } else {
        None
    };

    // Get remote objects
    let prefix = source_uri.key.clone().unwrap_or_default();
    let mut remote_files: HashMap<String, FileInfo> = HashMap::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let mut req = client
            .list_objects_v2()
            .bucket(&source_uri.bucket)
            .prefix(&prefix);

        if let Some(token) = &continuation_token {
            req = req.continuation_token(token);
        }

        let resp = req.send().await?;

        if let Some(contents) = resp.contents {
            for obj in contents {
                if let Some(key) = obj.key() {
                    let relative = key.strip_prefix(&prefix).unwrap_or(key);
                    let relative = relative.trim_start_matches('/');

                    if relative.is_empty() {
                        continue;
                    }

                    if !matches_patterns(relative, opts.include.as_deref(), opts.exclude.as_deref())? {
                        continue;
                    }

                    remote_files.insert(
                        relative.to_string(),
                        FileInfo {
                            size: obj.size().unwrap_or(0),
                            last_modified: obj.last_modified().map(|d| {
                                DateTime::<Utc>::from_timestamp(d.secs(), 0).unwrap_or_default()
                            }),
                        },
                    );
                }
            }
        }

        if resp.is_truncated.unwrap_or(false) {
            continuation_token = resp.next_continuation_token;
        } else {
            break;
        }
    }

    // Get local files
    let mut local_files: HashMap<String, FileInfo> = HashMap::new();
    if dest_path.exists() {
        for entry in WalkDir::new(dest_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                let relative = path
                    .strip_prefix(dest_path)
                    .unwrap_or(path)
                    .to_str()
                    .unwrap_or("")
                    .replace('\\', "/");

                if let Ok(metadata) = fs::metadata(path).await {
                    local_files.insert(
                        relative,
                        FileInfo {
                            size: metadata.len() as i64,
                            last_modified: metadata
                                .modified()
                                .ok()
                                .map(|t| DateTime::<Utc>::from(t)),
                        },
                    );
                }
            }
        }
    }

    if let Some(s) = spinner {
        s.finish_with_message(format!(
            "Remote: {} files, Local: {} files",
            remote_files.len(),
            local_files.len()
        ));
    }

    // Determine files to download
    let mut to_download: Vec<String> = Vec::new();
    for (relative, remote_info) in &remote_files {
        let needs_download = match local_files.get(relative) {
            None => true,
            Some(local_info) => {
                if opts.size_only {
                    remote_info.size != local_info.size
                } else {
                    remote_info.size != local_info.size
                        || remote_info.last_modified > local_info.last_modified
                }
            }
        };

        if needs_download {
            to_download.push(relative.clone());
        }
    }

    // Determine files to delete
    let mut to_delete: Vec<String> = Vec::new();
    if opts.delete {
        for relative in local_files.keys() {
            if !remote_files.contains_key(relative) {
                to_delete.push(relative.clone());
            }
        }
    }

    if !ctx.quiet {
        println!(
            "To download: {}, To delete: {}",
            to_download.len(),
            to_delete.len()
        );
    }

    // Download files
    let mut downloaded = 0;
    let mut download_bytes: u64 = 0;

    for relative in &to_download {
        let remote_key = if prefix.is_empty() {
            relative.clone()
        } else if prefix.ends_with('/') {
            format!("{}{}", prefix, relative)
        } else {
            format!("{}/{}", prefix, relative)
        };
        let local_path = dest_path.join(relative);

        if opts.dryrun {
            println!(
                "(dryrun) download: s3://{}/{} -> {}",
                source_uri.bucket,
                remote_key,
                local_path.display()
            );
            downloaded += 1;
            continue;
        }

        // Create parent directory
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let resp = client
            .get_object()
            .bucket(&source_uri.bucket)
            .key(&remote_key)
            .send()
            .await
            .context(format!("Failed to download {}", remote_key))?;

        let size = resp.content_length().unwrap_or(0) as u64;
        let mut file = fs::File::create(&local_path).await?;
        let mut stream = resp.body.into_async_read();

        let mut buf = [0u8; 8192];
        loop {
            use tokio::io::AsyncReadExt;
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n]).await?;
        }

        downloaded += 1;
        download_bytes += size;

        if !ctx.quiet {
            println!(
                "{}: s3://{}/{} -> {}",
                "download".green(),
                source_uri.bucket,
                remote_key,
                local_path.display()
            );
        }
    }

    // Delete local files
    let mut deleted = 0;
    if opts.delete {
        for relative in &to_delete {
            let local_path = dest_path.join(relative);

            if opts.dryrun {
                println!("(dryrun) delete: {}", local_path.display());
                deleted += 1;
                continue;
            }

            if local_path.exists() {
                fs::remove_file(&local_path).await?;
                deleted += 1;

                if !ctx.quiet {
                    println!("{}: {}", "delete".red(), local_path.display());
                }
            }
        }
    }

    if !ctx.quiet {
        println!(
            "\nSynced: {} downloaded ({}), {} deleted",
            downloaded,
            format_bytes(download_bytes),
            deleted
        );
    }

    Ok(())
}
