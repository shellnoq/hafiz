//! ls command - list buckets or objects

use super::CommandContext;
use crate::s3_client::{create_client, S3Uri};
use crate::utils::{format_datetime, format_size, format_storage_class};
use anyhow::Result;
use aws_sdk_s3::types::Object;
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::Serialize;

#[derive(Serialize)]
struct BucketInfo {
    name: String,
    creation_date: Option<String>,
}

#[derive(Serialize)]
struct ObjectInfo {
    key: String,
    size: i64,
    last_modified: Option<String>,
    storage_class: Option<String>,
    etag: Option<String>,
}

#[derive(Serialize)]
struct ListResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    buckets: Option<Vec<BucketInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    objects: Option<Vec<ObjectInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prefixes: Option<Vec<String>>,
    total_objects: usize,
    total_size: i64,
}

pub async fn execute(
    ctx: &CommandContext,
    path: &str,
    long: bool,
    human_readable: bool,
    recursive: bool,
    summarize: bool,
) -> Result<()> {
    let client = create_client(&ctx.config).await?;
    let uri = S3Uri::parse(path)?;

    if uri.bucket.is_empty() {
        // List buckets
        list_buckets(ctx, &client, long).await
    } else {
        // List objects
        list_objects(ctx, &client, &uri, long, human_readable, recursive, summarize).await
    }
}

async fn list_buckets(
    ctx: &CommandContext,
    client: &aws_sdk_s3::Client,
    long: bool,
) -> Result<()> {
    ctx.debug("Listing buckets...");

    let resp = client.list_buckets().send().await?;
    let buckets = resp.buckets();

    if ctx.is_json() {
        let bucket_infos: Vec<BucketInfo> = buckets
            .iter()
            .map(|b| BucketInfo {
                name: b.name().unwrap_or("").to_string(),
                creation_date: b.creation_date().map(|d| {
                    let secs = d.secs();
                    DateTime::<Utc>::from_timestamp(secs, 0)
                        .map(|dt| format_datetime(&dt))
                        .unwrap_or_default()
                }),
            })
            .collect();

        let result = ListResult {
            buckets: Some(bucket_infos),
            objects: None,
            prefixes: None,
            total_objects: buckets.len(),
            total_size: 0,
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        for bucket in buckets {
            let name = bucket.name().unwrap_or("");
            if long {
                let date = bucket.creation_date().map(|d| {
                    let secs = d.secs();
                    DateTime::<Utc>::from_timestamp(secs, 0)
                        .map(|dt| format_datetime(&dt))
                        .unwrap_or_default()
                });
                println!(
                    "{} {}",
                    date.unwrap_or_else(|| "                   ".to_string()),
                    name.blue().bold()
                );
            } else {
                println!("{}", name.blue().bold());
            }
        }

        if !ctx.quiet {
            println!("\nTotal: {} bucket(s)", buckets.len());
        }
    }

    Ok(())
}

async fn list_objects(
    ctx: &CommandContext,
    client: &aws_sdk_s3::Client,
    uri: &S3Uri,
    long: bool,
    human_readable: bool,
    recursive: bool,
    summarize: bool,
) -> Result<()> {
    ctx.debug(&format!(
        "Listing objects in bucket '{}' with prefix '{}'",
        uri.bucket,
        uri.key_or_empty()
    ));

    let prefix = uri.key.clone().unwrap_or_default();
    let delimiter = if recursive { None } else { Some("/".to_string()) };

    let mut continuation_token: Option<String> = None;
    let mut all_objects: Vec<Object> = Vec::new();
    let mut all_prefixes: Vec<String> = Vec::new();
    let mut total_size: i64 = 0;

    loop {
        let mut req = client
            .list_objects_v2()
            .bucket(&uri.bucket)
            .prefix(&prefix);

        if let Some(delim) = &delimiter {
            req = req.delimiter(delim);
        }

        if let Some(token) = &continuation_token {
            req = req.continuation_token(token);
        }

        let resp = req.send().await?;

        // Collect objects
        if let Some(contents) = resp.contents {
            for obj in contents {
                if let Some(size) = obj.size {
                    total_size += size;
                }
                all_objects.push(obj);
            }
        }

        // Collect common prefixes (directories)
        if let Some(prefixes) = resp.common_prefixes {
            for prefix in prefixes {
                if let Some(p) = prefix.prefix {
                    all_prefixes.push(p);
                }
            }
        }

        // Check for more results
        if resp.is_truncated.unwrap_or(false) {
            continuation_token = resp.next_continuation_token;
        } else {
            break;
        }
    }

    // Output results
    if ctx.is_json() {
        let object_infos: Vec<ObjectInfo> = all_objects
            .iter()
            .map(|o| ObjectInfo {
                key: o.key().unwrap_or("").to_string(),
                size: o.size().unwrap_or(0),
                last_modified: o.last_modified().map(|d| {
                    let secs = d.secs();
                    DateTime::<Utc>::from_timestamp(secs, 0)
                        .map(|dt| format_datetime(&dt))
                        .unwrap_or_default()
                }),
                storage_class: o.storage_class().map(|s| s.as_str().to_string()),
                etag: o.e_tag().map(|s| s.to_string()),
            })
            .collect();

        let result = ListResult {
            buckets: None,
            objects: Some(object_infos),
            prefixes: if all_prefixes.is_empty() {
                None
            } else {
                Some(all_prefixes)
            },
            total_objects: all_objects.len(),
            total_size,
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if summarize {
        // Summary only
        println!(
            "Total Objects: {}\nTotal Size: {}",
            all_objects.len(),
            format_size(total_size, human_readable)
        );
    } else {
        // Print prefixes (directories)
        for prefix in &all_prefixes {
            if long {
                println!("                   {:>12}  PRE {}", "", prefix.blue().bold());
            } else {
                println!("{}", prefix.blue().bold());
            }
        }

        // Print objects
        for obj in &all_objects {
            let key = obj.key().unwrap_or("");
            let size = obj.size().unwrap_or(0);

            if long {
                let date = obj.last_modified().map(|d| {
                    let secs = d.secs();
                    DateTime::<Utc>::from_timestamp(secs, 0)
                        .map(|dt| format_datetime(&dt))
                        .unwrap_or_default()
                });
                let storage = format_storage_class(obj.storage_class().map(|s| s.as_str()));

                println!(
                    "{} {:>12}  {:8}  {}",
                    date.unwrap_or_else(|| "                   ".to_string()),
                    format_size(size, human_readable),
                    storage,
                    key
                );
            } else {
                println!("{}", key);
            }
        }

        if !ctx.quiet {
            println!(
                "\nTotal: {} object(s), {}",
                all_objects.len(),
                format_size(total_size, human_readable)
            );
        }
    }

    Ok(())
}
