//! Hafiz CLI - S3-compatible storage command-line interface
//!
//! Usage:
//!   hafiz ls s3://bucket/prefix/
//!   hafiz cp local.txt s3://bucket/
//!   hafiz cp s3://bucket/file.txt ./
//!   hafiz sync ./local/ s3://bucket/prefix/
//!   hafiz mb s3://bucket
//!   hafiz rb s3://bucket
//!   hafiz rm s3://bucket/key

mod commands;
mod config;
mod progress;
mod s3_client;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "hafiz")]
#[command(author = "Hafiz Team")]
#[command(version = "0.1.0")]
#[command(about = "S3-compatible storage CLI", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Endpoint URL (e.g., http://localhost:9000)
    #[arg(long, env = "HAFIZ_ENDPOINT", global = true)]
    endpoint: Option<String>,

    /// Access key ID
    #[arg(long, env = "HAFIZ_ACCESS_KEY", global = true)]
    access_key: Option<String>,

    /// Secret access key
    #[arg(long, env = "HAFIZ_SECRET_KEY", global = true)]
    secret_key: Option<String>,

    /// AWS region
    #[arg(long, env = "HAFIZ_REGION", default_value = "us-east-1", global = true)]
    region: String,

    /// Configuration profile to use
    #[arg(long, short, env = "HAFIZ_PROFILE", global = true)]
    profile: Option<String>,

    /// Output format (text, json)
    #[arg(long, default_value = "text", global = true)]
    output: OutputFormat,

    /// Verbose output
    #[arg(long, short, global = true)]
    verbose: bool,

    /// Quiet mode (minimal output)
    #[arg(long, short, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// List buckets or objects
    #[command(alias = "list")]
    Ls {
        /// S3 path (s3://bucket or s3://bucket/prefix/)
        #[arg(default_value = "s3://")]
        path: String,

        /// Long listing format with details
        #[arg(long, short)]
        long: bool,

        /// Human-readable sizes
        #[arg(long, short = 'H')]
        human_readable: bool,

        /// Recursive listing
        #[arg(long, short)]
        recursive: bool,

        /// Show only summary
        #[arg(long)]
        summarize: bool,
    },

    /// Copy files to/from S3
    #[command(alias = "copy")]
    Cp {
        /// Source path (local or s3://bucket/key)
        source: String,

        /// Destination path (local or s3://bucket/key)
        destination: String,

        /// Recursive copy
        #[arg(long, short)]
        recursive: bool,

        /// Include pattern (glob)
        #[arg(long)]
        include: Option<String>,

        /// Exclude pattern (glob)
        #[arg(long)]
        exclude: Option<String>,

        /// Don't show progress
        #[arg(long)]
        no_progress: bool,

        /// Number of parallel transfers
        #[arg(long, default_value = "4")]
        parallel: usize,

        /// Storage class
        #[arg(long)]
        storage_class: Option<String>,

        /// Content type
        #[arg(long)]
        content_type: Option<String>,

        /// Dry run (show what would be copied)
        #[arg(long)]
        dryrun: bool,
    },

    /// Move files (copy + delete source)
    #[command(alias = "move")]
    Mv {
        /// Source path
        source: String,

        /// Destination path
        destination: String,

        /// Recursive move
        #[arg(long, short)]
        recursive: bool,

        /// Dry run
        #[arg(long)]
        dryrun: bool,
    },

    /// Sync directories
    Sync {
        /// Source path
        source: String,

        /// Destination path
        destination: String,

        /// Delete files in destination not in source
        #[arg(long)]
        delete: bool,

        /// Exclude pattern
        #[arg(long)]
        exclude: Option<String>,

        /// Include pattern
        #[arg(long)]
        include: Option<String>,

        /// Only sync if size differs
        #[arg(long)]
        size_only: bool,

        /// Dry run
        #[arg(long)]
        dryrun: bool,

        /// Number of parallel transfers
        #[arg(long, default_value = "4")]
        parallel: usize,
    },

    /// Remove objects
    #[command(alias = "remove", alias = "del", alias = "delete")]
    Rm {
        /// S3 path to remove
        path: String,

        /// Recursive delete
        #[arg(long, short)]
        recursive: bool,

        /// Force delete (no confirmation)
        #[arg(long, short)]
        force: bool,

        /// Include pattern
        #[arg(long)]
        include: Option<String>,

        /// Exclude pattern
        #[arg(long)]
        exclude: Option<String>,

        /// Dry run
        #[arg(long)]
        dryrun: bool,
    },

    /// Make bucket
    #[command(alias = "create-bucket")]
    Mb {
        /// Bucket name (s3://bucket-name)
        bucket: String,

        /// Region for bucket
        #[arg(long)]
        region: Option<String>,
    },

    /// Remove bucket
    #[command(alias = "delete-bucket")]
    Rb {
        /// Bucket name (s3://bucket-name)
        bucket: String,

        /// Force delete (delete all objects first)
        #[arg(long, short)]
        force: bool,
    },

    /// Get object info/metadata
    Head {
        /// S3 path
        path: String,
    },

    /// Generate presigned URL
    Presign {
        /// S3 path
        path: String,

        /// Expiration in seconds
        #[arg(long, default_value = "3600")]
        expires: u64,

        /// HTTP method (GET, PUT)
        #[arg(long, default_value = "GET")]
        method: String,
    },

    /// Manage configuration
    Configure {
        #[command(subcommand)]
        action: Option<ConfigureAction>,
    },

    /// Display bucket or object info
    Info {
        /// S3 path
        path: String,
    },

    /// Calculate disk usage
    Du {
        /// S3 path
        path: String,

        /// Human-readable sizes
        #[arg(long, short = 'H')]
        human_readable: bool,

        /// Summarize (show only total)
        #[arg(long, short)]
        summarize: bool,
    },

    /// Stream object content to stdout
    Cat {
        /// S3 path
        path: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigureAction {
    /// Set configuration value
    Set {
        /// Key to set
        key: String,
        /// Value to set
        value: String,
    },
    /// Get configuration value
    Get {
        /// Key to get
        key: String,
    },
    /// List all configuration
    List,
    /// Add a new profile
    AddProfile {
        /// Profile name
        name: String,
    },
    /// Remove a profile
    RemoveProfile {
        /// Profile name
        name: String,
    },
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let mut config = config::Config::load(cli.profile.as_deref())?;

    // Override with CLI args
    if let Some(endpoint) = cli.endpoint {
        config.endpoint = Some(endpoint);
    }
    if let Some(access_key) = cli.access_key {
        config.access_key = Some(access_key);
    }
    if let Some(secret_key) = cli.secret_key {
        config.secret_key = Some(secret_key);
    }
    config.region = cli.region;

    let ctx = commands::CommandContext {
        config,
        output_format: cli.output,
        verbose: cli.verbose,
        quiet: cli.quiet,
    };

    match cli.command {
        Commands::Ls {
            path,
            long,
            human_readable,
            recursive,
            summarize,
        } => {
            commands::ls::execute(&ctx, &path, long, human_readable, recursive, summarize).await
        }

        Commands::Cp {
            source,
            destination,
            recursive,
            include,
            exclude,
            no_progress,
            parallel,
            storage_class,
            content_type,
            dryrun,
        } => {
            commands::cp::execute(
                &ctx,
                &source,
                &destination,
                commands::cp::CpOptions {
                    recursive,
                    include,
                    exclude,
                    show_progress: !no_progress && !ctx.quiet,
                    parallel,
                    storage_class,
                    content_type,
                    dryrun,
                },
            )
            .await
        }

        Commands::Mv {
            source,
            destination,
            recursive,
            dryrun,
        } => commands::mv::execute(&ctx, &source, &destination, recursive, dryrun).await,

        Commands::Sync {
            source,
            destination,
            delete,
            exclude,
            include,
            size_only,
            dryrun,
            parallel,
        } => {
            commands::sync::execute(
                &ctx,
                &source,
                &destination,
                commands::sync::SyncOptions {
                    delete,
                    exclude,
                    include,
                    size_only,
                    dryrun,
                    parallel,
                },
            )
            .await
        }

        Commands::Rm {
            path,
            recursive,
            force,
            include,
            exclude,
            dryrun,
        } => {
            commands::rm::execute(
                &ctx,
                &path,
                commands::rm::RmOptions {
                    recursive,
                    force,
                    include,
                    exclude,
                    dryrun,
                },
            )
            .await
        }

        Commands::Mb { bucket, region } => commands::mb::execute(&ctx, &bucket, region).await,

        Commands::Rb { bucket, force } => commands::rb::execute(&ctx, &bucket, force).await,

        Commands::Head { path } => commands::head::execute(&ctx, &path).await,

        Commands::Presign {
            path,
            expires,
            method,
        } => commands::presign::execute(&ctx, &path, expires, &method).await,

        Commands::Configure { action } => commands::configure::execute(&ctx, action).await,

        Commands::Info { path } => commands::info::execute(&ctx, &path).await,

        Commands::Du {
            path,
            human_readable,
            summarize,
        } => commands::du::execute(&ctx, &path, human_readable, summarize).await,

        Commands::Cat { path } => commands::cat::execute(&ctx, &path).await,
    }
}
