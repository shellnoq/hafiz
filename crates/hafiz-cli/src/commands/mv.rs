//! mv command - move files (copy + delete)

use super::cp::{execute as cp_execute, CpOptions};
use super::rm::{execute as rm_execute, RmOptions};
use super::CommandContext;
use crate::s3_client::is_s3_uri;
use anyhow::Result;

pub async fn execute(
    ctx: &CommandContext,
    source: &str,
    destination: &str,
    recursive: bool,
    dryrun: bool,
) -> Result<()> {
    // First copy
    let cp_opts = CpOptions {
        recursive,
        include: None,
        exclude: None,
        show_progress: !ctx.quiet,
        parallel: 4,
        storage_class: None,
        content_type: None,
        dryrun,
    };

    cp_execute(ctx, source, destination, cp_opts).await?;

    // Then delete source (only if source is S3)
    if is_s3_uri(source) {
        let rm_opts = RmOptions {
            recursive,
            force: true,
            include: None,
            exclude: None,
            dryrun,
        };

        rm_execute(ctx, source, rm_opts).await?;
    } else if !dryrun {
        // Delete local source
        let path = std::path::Path::new(source);
        if path.is_file() {
            std::fs::remove_file(path)?;
        } else if path.is_dir() && recursive {
            std::fs::remove_dir_all(path)?;
        }
    }

    Ok(())
}
