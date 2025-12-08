//! Hafiz - Enterprise S3-Compatible Object Storage
//!
//! A high-performance, S3-compatible object storage server written in Rust.

use clap::{Parser, Subcommand};
use hafiz_core::config::HafizConfig;
use hafiz_s3_api::S3Server;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser)]
#[command(name = "hafiz")]
#[command(author = "Hafiz Team")]
#[command(version = hafiz_core::VERSION)]
#[command(about = "Enterprise S3-Compatible Object Storage", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Bind address
    #[arg(long, env = "HAFIZ_BIND_ADDRESS")]
    bind: Option<String>,

    /// Port number
    #[arg(short, long, env = "HAFIZ_PORT")]
    port: Option<u16>,

    /// Data directory
    #[arg(long, env = "HAFIZ_DATA_DIR")]
    data_dir: Option<String>,

    /// Root access key
    #[arg(long, env = "HAFIZ_ROOT_ACCESS_KEY")]
    access_key: Option<String>,

    /// Root secret key
    #[arg(long, env = "HAFIZ_ROOT_SECRET_KEY")]
    secret_key: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, env = "HAFIZ_LOG_LEVEL", default_value = "info")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the S3 server
    Server,
    
    /// Show version information
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    // Initialize logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&cli.log_level));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true))
        .with(filter)
        .init();

    // Load or create config
    let mut config = if let Some(config_path) = &cli.config {
        HafizConfig::from_file(config_path)?
    } else {
        HafizConfig::from_env()
    };

    // Override with CLI args
    if let Some(bind) = cli.bind {
        config.server.bind_address = bind;
    }
    if let Some(port) = cli.port {
        config.server.port = port;
    }
    if let Some(data_dir) = cli.data_dir {
        config.storage.data_dir = data_dir.into();
    }
    if let Some(access_key) = cli.access_key {
        config.auth.root_access_key = access_key;
    }
    if let Some(secret_key) = cli.secret_key {
        config.auth.root_secret_key = secret_key;
    }

    match cli.command {
        Some(Commands::Version) | None if cli.command.is_none() => {
            print_banner();
        }
        Some(Commands::Server) | None => {
            print_banner();
            run_server(config).await?;
        }
    }

    Ok(())
}

fn print_banner() {
    println!(r#"
    _   _                     ____  _                 
   | \ | | _____   ___   _ __|  _ \| |_ ___  _ __ ___ 
   |  \| |/ _ \ \ / / | | / __| |_) | __/ _ \| '__/ _ \
   | |\  | (_) \ V /| |_| \__ \  __/| || (_) | | |  __/
   |_| \_|\___/ \_/  \__,_|___/_|    \__\___/|_|  \___|
                                                       
   Enterprise S3-Compatible Object Storage
   Version: {}
"#, hafiz_core::VERSION);
}

async fn run_server(config: HafizConfig) -> anyhow::Result<()> {
    info!("Starting Hafiz server...");
    info!("Data directory: {:?}", config.storage.data_dir);
    info!("Database: {}", config.database.url);

    let server = S3Server::new(config);
    server.run().await?;

    Ok(())
}
