//! du command - calculate disk usage

use super::CommandContext;
use crate::s3_client::{create_client, S3Uri};
use crate::utils::format_size;
use anyhow::Result;
use colored::Colorize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct DuResult {
    path: String,
    size: i64,
    object_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    breakdown: Option<Vec<PrefixSize>>,
}

#[derive(Serialize)]
struct PrefixSize {
    prefix: String,
    size: i64,
    count: usize,
}

pub async fn execute(
    ctx: &CommandContext,
    path: &str,
    human_readable: bool,
    summarize: bool,
) -> Result<()> {
    let client = create_client(&ctx.config).await?;
    let uri = S3Uri::parse(path)?;
    let prefix = uri.key.clone().unwrap_or_default();

    ctx.debug(&format!(
        "Calculating disk usage for s3://{}/{}",
        uri.bucket, prefix
    ));

    // Track size by prefix (first level)
    let mut prefix_sizes: HashMap<String, (i64, usize)> = HashMap::new();
    let mut total_size: i64 = 0;
    let mut total_count: usize = 0;
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
                if let (Some(key), Some(size)) = (obj.key(), obj.size()) {
                    total_size += size;
                    total_count += 1;

                    // Get first-level prefix after the base prefix
                    let relative = key.strip_prefix(&prefix).unwrap_or(key);
                    let relative = relative.trim_start_matches('/');

                    if !summarize {
                        let first_part = if let Some(idx) = relative.find('/') {
                            format!("{}/", &relative[..idx])
                        } else {
                            relative.to_string()
                        };

                        let full_prefix = if prefix.is_empty() {
                            first_part
                        } else if prefix.ends_with('/') {
                            format!("{}{}", prefix, first_part)
                        } else {
                            format!("{}/{}", prefix, first_part)
                        };

                        let entry = prefix_sizes.entry(full_prefix).or_insert((0, 0));
                        entry.0 += size;
                        entry.1 += 1;
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

    if ctx.is_json() {
        let breakdown = if summarize {
            None
        } else {
            let mut items: Vec<PrefixSize> = prefix_sizes
                .into_iter()
                .map(|(p, (s, c))| PrefixSize {
                    prefix: p,
                    size: s,
                    count: c,
                })
                .collect();
            items.sort_by(|a, b| b.size.cmp(&a.size));
            Some(items)
        };

        let result = DuResult {
            path: path.to_string(),
            size: total_size,
            object_count: total_count,
            breakdown,
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if !summarize {
            // Sort by size descending
            let mut items: Vec<_> = prefix_sizes.into_iter().collect();
            items.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

            for (prefix, (size, count)) in items {
                println!(
                    "{:>12}  {:>8} obj  s3://{}/{}",
                    format_size(size, human_readable),
                    count,
                    uri.bucket,
                    prefix
                );
            }

            println!();
        }

        println!(
            "{:>12}  {:>8} obj  {} (total)",
            format_size(total_size, human_readable).bold(),
            total_count,
            path.blue()
        );
    }

    Ok(())
}
