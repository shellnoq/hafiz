//! cp command - copy files to/from S3

use super::CommandContext;
use crate::progress::{create_spinner, create_transfer_progress, format_bytes};
use crate::s3_client::{create_client, is_s3_uri, S3Uri, TransferDirection};
use crate::utils::{determine_dest_key, guess_content_type, matches_patterns};
use anyhow::{Context, Result};
use aws_sdk_s3::primitives::ByteStream;
use colored::Colorize;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;

pub struct CpOptions {
    pub recursive: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub show_progress: bool,
    pub parallel: usize,
    pub storage_class: Option<String>,
    pub content_type: Option<String>,
    pub dryrun: bool,
}

pub async fn execute(
    ctx: &CommandContext,
    source: &str,
    destination: &str,
    opts: CpOptions,
) -> Result<()> {
    let direction = TransferDirection::determine(source, destination);

    match direction {
        TransferDirection::Upload => upload(ctx, source, destination, &opts).await,
        TransferDirection::Download => download(ctx, source, destination, &opts).await,
        TransferDirection::S3ToS3 => s3_copy(ctx, source, destination, &opts).await,
        TransferDirection::LocalToLocal => {
            anyhow::bail!("Local to local copy is not supported. Use system cp command.")
        }
    }
}

async fn upload(
    ctx: &CommandContext,
    source: &str,
    destination: &str,
    opts: &CpOptions,
) -> Result<()> {
    let client = create_client(&ctx.config).await?;
    let dest_uri = S3Uri::parse(destination)?;
    let source_path = Path::new(source);

    if !source_path.exists() {
        anyhow::bail!("Source path does not exist: {}", source);
    }

    if source_path.is_file() {
        // Single file upload
        upload_file(ctx, &client, source_path, &dest_uri, opts).await?;
    } else if source_path.is_dir() {
        // Directory upload
        if !opts.recursive {
            anyhow::bail!("Cannot copy directory without --recursive flag");
        }
        upload_directory(ctx, &client, source_path, &dest_uri, opts).await?;
    } else {
        anyhow::bail!("Source is neither a file nor a directory: {}", source);
    }

    Ok(())
}

async fn upload_file(
    ctx: &CommandContext,
    client: &aws_sdk_s3::Client,
    source: &Path,
    dest_uri: &S3Uri,
    opts: &CpOptions,
) -> Result<()> {
    let filename = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    let dest_key = determine_dest_key(
        source.to_str().unwrap_or(""),
        dest_uri.key.as_deref(),
        dest_uri.is_prefix(),
    );

    // Check patterns
    if !matches_patterns(&dest_key, opts.include.as_deref(), opts.exclude.as_deref())? {
        ctx.debug(&format!("Skipping {} (pattern mismatch)", filename));
        return Ok(());
    }

    if opts.dryrun {
        println!(
            "(dryrun) upload: {} -> s3://{}/{}",
            source.display(),
            dest_uri.bucket,
            dest_key
        );
        return Ok(());
    }

    ctx.debug(&format!(
        "Uploading {} to s3://{}/{}",
        source.display(),
        dest_uri.bucket,
        dest_key
    ));

    let metadata = fs::metadata(source).await?;
    let file_size = metadata.len();

    let content_type = opts
        .content_type
        .clone()
        .unwrap_or_else(|| guess_content_type(source.to_str().unwrap_or("")));

    let body = ByteStream::from_path(source)
        .await
        .context("Failed to read file")?;

    let progress = if opts.show_progress {
        Some(create_transfer_progress(file_size, filename))
    } else {
        None
    };

    let mut req = client
        .put_object()
        .bucket(&dest_uri.bucket)
        .key(&dest_key)
        .content_type(content_type)
        .body(body);

    if let Some(storage_class) = &opts.storage_class {
        req = req.storage_class(storage_class.as_str().into());
    }

    req.send().await.context("Upload failed")?;

    if let Some(pb) = progress {
        pb.finish_with_message("Done");
    }

    if !ctx.quiet {
        println!(
            "{}: {} -> s3://{}/{}",
            "upload".green(),
            source.display(),
            dest_uri.bucket,
            dest_key
        );
    }

    Ok(())
}

async fn upload_directory(
    ctx: &CommandContext,
    client: &aws_sdk_s3::Client,
    source: &Path,
    dest_uri: &S3Uri,
    opts: &CpOptions,
) -> Result<()> {
    let spinner = if opts.show_progress && !ctx.quiet {
        Some(create_spinner("Scanning directory..."))
    } else {
        None
    };

    let mut files: Vec<(std::path::PathBuf, String)> = Vec::new();

    for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path().to_path_buf();
            let relative = path
                .strip_prefix(source)
                .unwrap_or(&path)
                .to_str()
                .unwrap_or("")
                .replace('\\', "/"); // Windows compatibility

            let dest_key = if let Some(prefix) = &dest_uri.key {
                if prefix.ends_with('/') {
                    format!("{}{}", prefix, relative)
                } else {
                    format!("{}/{}", prefix, relative)
                }
            } else {
                relative.clone()
            };

            // Check patterns
            if matches_patterns(&relative, opts.include.as_deref(), opts.exclude.as_deref())? {
                files.push((path, dest_key));
            }
        }
    }

    if let Some(s) = spinner {
        s.finish_with_message(format!("Found {} files", files.len()));
    }

    let total_files = files.len();
    let mut uploaded = 0;
    let mut total_bytes: u64 = 0;

    for (path, dest_key) in files {
        if opts.dryrun {
            println!(
                "(dryrun) upload: {} -> s3://{}/{}",
                path.display(),
                dest_uri.bucket,
                dest_key
            );
            uploaded += 1;
            continue;
        }

        let metadata = fs::metadata(&path).await?;
        let file_size = metadata.len();

        let content_type = opts
            .content_type
            .clone()
            .unwrap_or_else(|| guess_content_type(path.to_str().unwrap_or("")));

        let body = ByteStream::from_path(&path).await?;

        let mut req = client
            .put_object()
            .bucket(&dest_uri.bucket)
            .key(&dest_key)
            .content_type(content_type)
            .body(body);

        if let Some(storage_class) = &opts.storage_class {
            req = req.storage_class(storage_class.as_str().into());
        }

        req.send().await?;

        uploaded += 1;
        total_bytes += file_size;

        if !ctx.quiet {
            println!(
                "{}: {} -> s3://{}/{} [{}/{}]",
                "upload".green(),
                path.display(),
                dest_uri.bucket,
                dest_key,
                uploaded,
                total_files
            );
        }
    }

    if !ctx.quiet && !opts.dryrun {
        println!(
            "\nUploaded {} file(s), {}",
            uploaded,
            format_bytes(total_bytes)
        );
    }

    Ok(())
}

async fn download(
    ctx: &CommandContext,
    source: &str,
    destination: &str,
    opts: &CpOptions,
) -> Result<()> {
    let client = create_client(&ctx.config).await?;
    let source_uri = S3Uri::parse(source)?;
    let dest_path = Path::new(destination);

    if source_uri.is_prefix() || opts.recursive {
        // Download multiple objects
        download_prefix(ctx, &client, &source_uri, dest_path, opts).await
    } else {
        // Download single object
        download_object(ctx, &client, &source_uri, dest_path, opts).await
    }
}

async fn download_object(
    ctx: &CommandContext,
    client: &aws_sdk_s3::Client,
    source_uri: &S3Uri,
    dest_path: &Path,
    opts: &CpOptions,
) -> Result<()> {
    let key = source_uri.key.as_ref().context("Object key required")?;

    // Determine final destination path
    let final_path = if dest_path.is_dir() || destination_is_directory(dest_path) {
        let filename = key.rsplit('/').next().unwrap_or(key);
        dest_path.join(filename)
    } else {
        dest_path.to_path_buf()
    };

    if opts.dryrun {
        println!(
            "(dryrun) download: s3://{}/{} -> {}",
            source_uri.bucket,
            key,
            final_path.display()
        );
        return Ok(());
    }

    ctx.debug(&format!(
        "Downloading s3://{}/{} to {}",
        source_uri.bucket,
        key,
        final_path.display()
    ));

    // Create parent directory if needed
    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let resp = client
        .get_object()
        .bucket(&source_uri.bucket)
        .key(key)
        .send()
        .await
        .context("Download failed")?;

    let content_length = resp.content_length().unwrap_or(0) as u64;

    let progress = if opts.show_progress {
        Some(create_transfer_progress(
            content_length,
            final_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file"),
        ))
    } else {
        None
    };

    let mut file = fs::File::create(&final_path).await?;
    let mut stream = resp.body.into_async_read();
    let mut downloaded: u64 = 0;

    let mut buf = [0u8; 8192];
    loop {
        use tokio::io::AsyncReadExt;
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).await?;
        downloaded += n as u64;

        if let Some(pb) = &progress {
            pb.set_position(downloaded);
        }
    }

    if let Some(pb) = progress {
        pb.finish_with_message("Done");
    }

    if !ctx.quiet {
        println!(
            "{}: s3://{}/{} -> {}",
            "download".green(),
            source_uri.bucket,
            key,
            final_path.display()
        );
    }

    Ok(())
}

async fn download_prefix(
    ctx: &CommandContext,
    client: &aws_sdk_s3::Client,
    source_uri: &S3Uri,
    dest_path: &Path,
    opts: &CpOptions,
) -> Result<()> {
    let prefix = source_uri.key.clone().unwrap_or_default();

    let spinner = if opts.show_progress && !ctx.quiet {
        Some(create_spinner("Listing objects..."))
    } else {
        None
    };

    // List all objects
    let mut objects: Vec<(String, i64)> = Vec::new();
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
                if let (Some(key), Some(size)) = (obj.key(), obj.size()) {
                    // Check patterns
                    if matches_patterns(key, opts.include.as_deref(), opts.exclude.as_deref())? {
                        objects.push((key.to_string(), size));
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

    if let Some(s) = spinner {
        s.finish_with_message(format!("Found {} objects", objects.len()));
    }

    let total_objects = objects.len();
    let mut downloaded = 0;
    let mut total_bytes: u64 = 0;

    for (key, size) in objects {
        // Calculate relative path
        let relative = key.strip_prefix(&prefix).unwrap_or(&key);
        let relative = relative.trim_start_matches('/');
        let final_path = dest_path.join(relative);

        if opts.dryrun {
            println!(
                "(dryrun) download: s3://{}/{} -> {}",
                source_uri.bucket,
                key,
                final_path.display()
            );
            downloaded += 1;
            continue;
        }

        // Create parent directory
        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let resp = client
            .get_object()
            .bucket(&source_uri.bucket)
            .key(&key)
            .send()
            .await?;

        let mut file = fs::File::create(&final_path).await?;
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
        total_bytes += size as u64;

        if !ctx.quiet {
            println!(
                "{}: s3://{}/{} -> {} [{}/{}]",
                "download".green(),
                source_uri.bucket,
                key,
                final_path.display(),
                downloaded,
                total_objects
            );
        }
    }

    if !ctx.quiet && !opts.dryrun {
        println!(
            "\nDownloaded {} file(s), {}",
            downloaded,
            format_bytes(total_bytes)
        );
    }

    Ok(())
}

async fn s3_copy(
    ctx: &CommandContext,
    source: &str,
    destination: &str,
    opts: &CpOptions,
) -> Result<()> {
    let client = create_client(&ctx.config).await?;
    let source_uri = S3Uri::parse(source)?;
    let dest_uri = S3Uri::parse(destination)?;

    let source_key = source_uri.key.as_ref().context("Source key required")?;
    let dest_key = determine_dest_key(source_key, dest_uri.key.as_deref(), dest_uri.is_prefix());

    if opts.dryrun {
        println!(
            "(dryrun) copy: s3://{}/{} -> s3://{}/{}",
            source_uri.bucket, source_key, dest_uri.bucket, dest_key
        );
        return Ok(());
    }

    ctx.debug(&format!(
        "Copying s3://{}/{} to s3://{}/{}",
        source_uri.bucket, source_key, dest_uri.bucket, dest_key
    ));

    let copy_source = format!("{}/{}", source_uri.bucket, source_key);

    let mut req = client
        .copy_object()
        .bucket(&dest_uri.bucket)
        .key(&dest_key)
        .copy_source(&copy_source);

    if let Some(storage_class) = &opts.storage_class {
        req = req.storage_class(storage_class.as_str().into());
    }

    req.send().await.context("Copy failed")?;

    if !ctx.quiet {
        println!(
            "{}: s3://{}/{} -> s3://{}/{}",
            "copy".green(),
            source_uri.bucket,
            source_key,
            dest_uri.bucket,
            dest_key
        );
    }

    Ok(())
}

fn destination_is_directory(path: &Path) -> bool {
    // Check if path ends with / or is an existing directory
    path.to_str()
        .map(|s| s.ends_with('/') || s.ends_with('\\'))
        .unwrap_or(false)
        || path.is_dir()
}
