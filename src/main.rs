mod model;
mod util;

use crate::{
    model::config::Config,
    util::{extract_number_after_double_arrow, get_nth_file},
};
use directories::ProjectDirs;
use log::{debug, error, info, warn};
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;
use tokio::{
    io::{self},
    net::UnixStream,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    env_logger::init();
    std::env::set_var("RUST_LOG", "warn");

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
        std::env::set_var("RUST_LOG", "debug");
        std::env::set_var("RUST_BACKTRACE", "full");
    }

    // Get the socket path from the environment variable
    let hyprland_instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")?;

    // Build the socket path with appropriate format
    let socket_path =
        Path::new("/tmp/hypr/").join(format!("{}/.socket2.sock", hyprland_instance_signature));
    info!("Socket path: {:?}", socket_path);

    // Connect to the socket using UnixStream
    let stream = UnixStream::connect(socket_path).await?;

    handle_loop(stream, &config).await?;

    Ok(())
}

async fn handle_loop(
    stream: UnixStream,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    const LINE_ENDING: &str = "\n";
    let mut buffer = vec![0; 50]; // Adjust buffer size as needed
    let mut line_buffer = String::new();

    loop {
        match stream.try_read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    // Handle connection closed
                    warn!("\n[!!]:Connection closed");
                    break;
                }

                let data_str = from_utf8(&buffer[..n])?;
                line_buffer.push_str(data_str);

                // Check for line endings within the buffer
                let mut lines = line_buffer.split(LINE_ENDING);

                while let Some(line) = lines.next() {
                    if !line.is_empty() {
                        debug!("\nHandling event: {}", line);
                        let _ = handle_event(line, &config).await;
                    }
                }

                // Remove processed lines from the buffer
                line_buffer = lines.next().unwrap_or("").to_string(); // Keep any remaining data
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Continue reading in next iteration
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
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

                    let output = Command::new("hyprctl")
                        .args(["hyprpaper", "wallpaper", &command_str])
                        .output();

                    match output {
                        Ok(output) => {
                            debug!("Command output: {:?}", output);

                            match output.status.success() {
                                true => {
                                    let result = String::from_utf8(output.stdout)
                                        .expect("Invalid UTF-8 sequence");

                                    if result.contains("wallpaper failed (not preloaded)")
                                        || result.contains(&format!(
                                            "Couldn't connect to /tmp/hypr/{}/.hyprpaper.sock",
                                            env::var("HYPRLAND_INSTANCE_SIGNATURE")?
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
