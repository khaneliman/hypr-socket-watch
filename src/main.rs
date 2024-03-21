use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::from_utf8;
use std::{fs, process::Command};

use tokio::{
    io::{self, Interest},
    net::UnixStream,
};

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub monitor: String,
    pub wallpapers: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the socket path from the environment variable
    let hyprland_instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")?;

    // Build the socket path with appropriate format
    let socket_path =
        Path::new("/tmp/hypr/").join(format!("{}/.socket2.sock", hyprland_instance_signature));
    println!("Socket path: {:?}", socket_path);

    // Connect to the socket using UnixStream
    let stream = UnixStream::connect(socket_path).await?;

    loop {
        println!("Starting loop");
        let ready = stream
            .ready(Interest::READABLE | Interest::WRITABLE)
            .await?;
        println!("{:?}", ready);

        if ready.is_readable() {
            let mut data = vec![0; 400];
            println!("Reading data");
            // Try to read data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match stream.try_read(&mut data) {
                Ok(_) => {
                    println!("Read data: {:?}", data);
                    if let Ok(line) = String::from_utf8(data) {
                        let _ = handle(&line).await;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }
}

async fn handle(line: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading config...");
    let mut config_file = File::open("config.yaml")?;
    let mut config_str = String::new();
    config_file.read_to_string(&mut config_str)?;

    let config: Config = serde_yaml::from_str(&config_str).expect("error getting config");

    match line {
        line if line.starts_with("monitoradded") => println!("Monitor added event: {}", line),
        line if line.starts_with("focusedmon") => println!("Focused monitor event: {}", line),
        line if line.starts_with("workspace") && !line.starts_with("workspacev2") => {
            println!("Workspace event: {}", line);
            let n = extract_number_after_double_arrow(line.trim());
            let wallpaper = get_nth_file(&config.wallpapers, n.expect("Expected number"));

            let command_str = format!(
                "{},{}",
                config.monitor,
                wallpaper.expect("Expected wallpaper path")
            );

            println!("Command: {}", &command_str);

            let output = Command::new("hyprctl")
                .args(["hyprpaper", "wallpaper", &command_str])
                .output()
                .expect("Failed to execute command");

            if output.status.success() {
                let result = String::from_utf8(output.stdout).expect("Invalid UTF-8 sequence");
                println!("Command output: {}", result);
            } else {
                let error = String::from_utf8(output.stderr).expect("Invalid UTF-8 sequence");
                println!("Error executing command: {}", error);
            }
        }
        _ => println!("Unknown event: {}", line),
    }

    return Ok(());
}

fn extract_after_double_arrow(input_string: &str) -> Option<String> {
    println!("String Extract Input string: {}**", input_string);
    // Split the input string by "\n"
    let parts: Vec<&str> = input_string.split("\n").collect();

    // Extract the first part before the "\n" character
    if let Some(first_part) = parts.first() {
        // Trim any extra spaces
        let cleaned_string = first_part.trim();

        // Split the cleaned string by ">>" (maximum of one split)
        let parts: Vec<&str> = cleaned_string.splitn(2, ">>").collect();
        println!("Parts: {:?}", parts);

        // Check if there's a part after the double arrow
        if parts.len() > 1 {
            println!("Parts length: {}", parts.len());
            return Some(parts[1].to_string());
        }
    }

    // No part after double arrow, return None
    None
}

fn extract_number_after_double_arrow(input_string: &str) -> Option<u32> {
    // Extract the part after the double arrow
    println!("Number Extract Input string: {}**", input_string);
    // let decoded_string = from_utf8(&input_string.as_bytes()).unwrap_or(input_string);
    let decoded_string = from_utf8(&input_string.as_bytes()).unwrap_or(input_string);
    let trimmed_string = decoded_string.trim_end_matches(char::from(0));
    // let part_after_arrow = extract_after_double_arrow(&trimmed_string)?;

    println!(
        "Number Extract Input string (decoded): {}**",
        decoded_string
    );

    if let Some(part_after_arrow) = extract_after_double_arrow(trimmed_string) {
        println!("Part after arrow: {}", part_after_arrow);

        // Parse the extracted part as a u32
        match part_after_arrow.trim().parse::<u32>() {
            Ok(number) => {
                println!("Number: {}", number);
                return Some(number);
            }
            Err(_) => {
                // Handle the error if the part is not a valid number
                println!("Error: Invalid number format: {}", part_after_arrow);
            }
        }
    }

    None // No number found
}

// Parse arguments using std::env
fn parse_args() -> Option<(String, u32)> {
    let mut args = std::env::args().skip(1);
    if let Some(directory) = args.next() {
        if let Some(n_str) = args.next() {
            let n = n_str.parse::<u32>();
            if n.is_ok() {
                return Some((directory, n.unwrap()));
            } else {
                println!("Error: Invalid number for n: {}", n_str);
            }
        }
    }
    None
}

fn get_nth_file(directory: &str, n: u32) -> Option<String> {
    println!("Directory: {}", directory);
    // Check if directory exists
    match !Path::is_dir(Path::new(directory)) {
        true => {
            println!("Error: Directory '{}' does not exist.", directory);
            return None;
        }
        false => (),
    }

    // Get list of files, sorted
    let mut files = fs::read_dir(directory)
        .unwrap()
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.file_name())
        .collect::<Vec<_>>();
    files.sort();

    let index = if n as usize > files.len() {
        0 // Default to the first file
    } else {
        n as usize - 1
    };
    // Get the nth file name
    let file_name = &files[index];

    // Get full path
    let full_path = format!("{}{}", directory, file_name.to_str().unwrap());

    println!("File: {}", full_path);

    Some(full_path)
}
