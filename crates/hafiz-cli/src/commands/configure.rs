//! configure command - manage configuration

use super::CommandContext;
use crate::config::Config;
use crate::ConfigureAction;
use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

pub async fn execute(ctx: &CommandContext, action: Option<ConfigureAction>) -> Result<()> {
    match action {
        Some(ConfigureAction::Set { key, value }) => set_config(&key, &value),
        Some(ConfigureAction::Get { key }) => get_config(&key),
        Some(ConfigureAction::List) => list_config(),
        Some(ConfigureAction::AddProfile { name }) => add_profile(&name),
        Some(ConfigureAction::RemoveProfile { name }) => remove_profile(&name),
        None => interactive_configure(),
    }
}

fn set_config(key: &str, value: &str) -> Result<()> {
    let mut config = Config::load(None)?;
    config.set_value(key, value)?;
    config.save(None)?;
    println!("Set {} = {}", key.cyan(), value);
    Ok(())
}

fn get_config(key: &str) -> Result<()> {
    let config = Config::load(None)?;
    match config.get_value(key) {
        Some(value) => println!("{}", value),
        None => println!("(not set)"),
    }
    Ok(())
}

fn list_config() -> Result<()> {
    let config = Config::load(None)?;

    println!("{}", "Current configuration:".bold());
    println!();

    for key in Config::keys() {
        let value = config.get_value(key).unwrap_or_else(|| "(not set)".to_string());
        println!("  {}: {}", key.cyan(), value);
    }

    println!();
    println!("{}", "Available profiles:".bold());

    let profiles = Config::list_profiles()?;
    if profiles.is_empty() {
        println!("  (none)");
    } else {
        for profile in profiles {
            println!("  - {}", profile);
        }
    }

    println!();
    println!(
        "Config file: {}",
        Config::config_path()?.display().to_string().dimmed()
    );

    Ok(())
}

fn add_profile(name: &str) -> Result<()> {
    let config = Config::default();
    config.save(Some(name))?;
    println!("Created profile: {}", name.green());
    println!("Use 'hafiz configure set <key> <value> --profile {}' to configure it.", name);
    Ok(())
}

fn remove_profile(name: &str) -> Result<()> {
    Config::delete_profile(name)?;
    println!("Removed profile: {}", name.red());
    Ok(())
}

fn interactive_configure() -> Result<()> {
    println!("{}", "Hafiz CLI Configuration".bold());
    println!("Press Enter to keep current value.\n");

    let mut config = Config::load(None).unwrap_or_default();

    // Endpoint
    let current_endpoint = config.endpoint.as_deref().unwrap_or("");
    print!("Endpoint URL [{}]: ", current_endpoint);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if !input.is_empty() {
        config.endpoint = Some(input.to_string());
    }

    // Access Key
    let current_access = config.access_key.as_deref().unwrap_or("");
    print!("Access Key [{}]: ", current_access);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if !input.is_empty() {
        config.access_key = Some(input.to_string());
    }

    // Secret Key
    let masked = if config.secret_key.is_some() {
        "***"
    } else {
        ""
    };
    print!("Secret Key [{}]: ", masked);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if !input.is_empty() {
        config.secret_key = Some(input.to_string());
    }

    // Region
    print!("Region [{}]: ", config.region);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if !input.is_empty() {
        config.region = input.to_string();
    }

    // Save
    config.save(None)?;

    println!();
    println!(
        "{} Configuration saved to {}",
        "✓".green(),
        Config::config_path()?.display()
    );

    Ok(())
}
