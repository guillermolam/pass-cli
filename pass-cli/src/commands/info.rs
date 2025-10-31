use anyhow::{Context, Result};
use pass::PassClient;
use std::path::PathBuf;

use crate::commands::update;

pub async fn run(client: PassClient, base_dir: PathBuf) -> Result<()> {
    let info = client.get_info().await.context("Error getting user info")?;

    // Only show ENV if it's not "prod"
    let env_str = format!("{:?}", info.env);
    if env_str != "Prod" {
        println!("- ENV: {}", env_str);
    }

    // Show release track
    let release_track = update::get_release_track(&base_dir)
        .await
        .unwrap_or_else(|_| "stable".to_string());
    println!("- Release track: {}", release_track);

    println!("- ID: {}", info.user.id);
    println!("- Username: {}", info.user.name);
    println!("- Email: {}", info.user.email);
    Ok(())
}
