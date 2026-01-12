use crate::commands::update::InstallSource;
use crate::commands::{OutputFormat, settings_helper, update};
use crate::telemetry::event::CommandEvent;
use anyhow::{Context, Result};
use pass::PassClient;
use std::path::PathBuf;

#[derive(serde::Serialize)]
struct InfoOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    pub release_track: String,
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_source: Option<String>,
}

pub async fn run(
    client: PassClient,
    base_dir: PathBuf,
    output: Option<OutputFormat>,
) -> Result<()> {
    client.emit_telemetry(&CommandEvent::new("info")).await;

    // Resolve output format from settings if not provided
    let output = match output {
        Some(fmt) => fmt,
        None => settings_helper::get_default_format(&client)
            .await?
            .unwrap_or(OutputFormat::Human),
    };

    let info = client.get_info().await.context("Error getting user info")?;

    // Only show ENV if it's not "prod"
    let env_str = format!("{:?}", info.env);
    let env = if env_str != "Prod" {
        Some(env_str)
    } else {
        None
    };

    // Show release track
    let release_track = update::get_release_track(&base_dir)
        .await
        .unwrap_or_else(|_| "stable".to_string());

    let install_source = update::get_install_source()?;
    let install_source_str = if install_source != InstallSource::Standard {
        Some(format!("{:?}", install_source))
    } else {
        None
    };

    let info_output = InfoOutput {
        env,
        release_track,
        id: info.user.id,
        username: info.user.name,
        email: info.user.email,
        install_source: install_source_str,
    };

    print(info_output, output).context("Error printing info")?;

    Ok(())
}

fn print(info: InfoOutput, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Human => {
            if let Some(env) = &info.env {
                println!("- ENV: {}", env);
            }
            println!("- Release track: {}", info.release_track);
            println!("- ID: {}", info.id);
            println!("- Username: {}", info.username);
            println!("- Email: {}", info.email);
            if let Some(install_source) = &info.install_source {
                println!("- Install source: {}", install_source);
            }
        }
        OutputFormat::Json => {
            let as_json = serde_json::to_string_pretty(&info).context("Error serializing info")?;
            println!("{as_json}");
        }
    }

    Ok(())
}
