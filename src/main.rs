mod model;
mod util;

use crate::{
    model::config::Config,
    util::{extract_number_after_double_arrow, get_nth_file},
};
use directories::ProjectDirs;
use log::{debug, error, info};
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tokio::{io::AsyncBufReadExt, net::UnixStream, process::Command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut binding = env_logger::builder();
    let logger = binding
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp(None);

    info!("Loading config...");
    let proj_dirs = ProjectDirs::from("com", "khaneliman", "hypr-socket-watch");
    let config_path = proj_dirs
        .expect("No config found")
        .config_dir()
        .join("config.yaml");

    let mut config_file = File::open(&config_path)?;
    let mut config_str = String::new();
    config_file.read_to_string(&mut config_str)?;

    let config: Config = serde_yaml::from_str(&config_str).expect("error getting config");

    if config.debug.is_some() && config.debug.unwrap() {
        std::env::set_var("RUST_BACKTRACE", "full");
        logger.filter_level(log::LevelFilter::Debug);
    }

    // Initialize the logger
    logger.init();

    // Get the socket path from the environment variable
    let hyprland_instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR")?;

    // Build the socket path with appropriate format
    let path = format!(
        "{}/hypr/{}/.socket2.sock",
        xdg_runtime_dir, hyprland_instance_signature
    );
    let socket_path = Path::new(&path);
    info!("Socket path: {:?}", socket_path);

    // Connect to the socket using UnixStream
    let stream = UnixStream::connect(socket_path).await?;

    let reader = tokio::io::BufReader::new(stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if !line.is_empty() {
            debug!("\nHandling event: {}", line);
            let _ = handle_event(&line, &config).await;
        }
    }

    Ok(())
}

async fn handle_event(line: &str, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let workspace_regex = Regex::new(r"^workspace>>\d+").unwrap();

    match line {
        line if line.starts_with("monitoradded") => debug!("\nMonitor added event: {}", line),
        line if line.starts_with("focusedmon") => debug!("\nFocused monitor event: {}", line),
        line if workspace_regex.is_match(line) => {
            debug!("\nWorkspace event: {}", line);
            let n = extract_number_after_double_arrow(line.trim());
            let wallpaper = get_nth_file(&config.wallpapers, n.expect("Expected number"));

            match wallpaper {
                Ok(wallpaper) => {
                    let command_str = format!("{},{}", config.monitor, wallpaper.display());

                    debug!("Command: hyprctl hyprpaper wallpaper {}", &command_str);
                    tokio::spawn(async move {
                        let output = Command::new("hyprctl")
                            .args(["hyprpaper", "wallpaper", &command_str])
                            .output()
                            .await;

                        match output {
                            Ok(output) => {
                                debug!("Command output: {:?}", output);

                                match output.status.success() {
                                    true => {
                                        let result = String::from_utf8(output.stdout)
                                            .expect("Invalid UTF-8 sequence");

                                        if result.contains("wallpaper failed (not preloaded)")
                                            || result.contains(&format!(
                                                "Couldn't connect to /tmp/hypr/{:?}/.hyprpaper.sock",
                                                env::var("HYPRLAND_INSTANCE_SIGNATURE")
                                            ))
                                        {
                                            error!("Wallpaper setting failed: {}", result);
                                        } else {
                                            debug!("Command output: {}", result);
                                        }
                                    }
                                    false => {
                                        let error = String::from_utf8(output.stderr)
                                            .expect("Invalid UTF-8 sequence");
                                        error!("Error executing command: {}", error);
                                    }
                                }
                            }
                            Err(error) => {
                                error!("Error executing command: {}", error);
                            }
                        }
                    });
                }
                Err(error) => {
                    error!("Error getting wallpaper: {}", error);
                }
            }
        }
        _ => {
            debug!("\nIgnored event: {}", line)
        }
    }

    return Ok(());
}
