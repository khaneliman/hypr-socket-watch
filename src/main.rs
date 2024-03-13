use std::env;
use std::path::Path;
use std::str::from_utf8;
use std::{fs, process::Command};

use tokio::{
    io::{self, Interest},
    net::UnixStream,
};

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
        let ready = stream
            .ready(Interest::READABLE | Interest::WRITABLE)
            .await?;

        if ready.is_readable() {
            let mut data = vec![0; 65];
            // Try to read data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match stream.try_read(&mut data) {
                Ok(_) => {
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
    match line {
        line if line.starts_with("monitoradded") => println!("Monitor added event: {}", line),
        line if line.starts_with("focusedmon") => println!("Focused monitor event: {}", line),
        line if line.starts_with("workspace") && !line.starts_with("workspacev2") => {
            println!("Workspace event: {}", line);
            // NOTE: throws socket error communicating with hyprpaper
            let n = extract_number_after_double_arrow(line.trim());
            // TODO: replace with logic to get wallpaper path
            let wallpaper = get_nth_file("/nix/store/xl4p5kciyn2kahc3kpafvgjwqlj0q8yy-khanelinix.wallpapers/share/wallpapers/", n.expect("Expected number"));
            let command_str = format!("DP-1,{}", wallpaper.expect("Expected wallpaper path"));

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

            // let hyprland_instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
            //
            // // Build the socket path with appropriate format
            // let socket_path = Path::new("/tmp/hypr/")
            //     .join(format!("{}/.hyprpaper.sock", hyprland_instance_signature));
            // println!("Socket path: {:?}", socket_path);
            //
            // // Connect to the socket using UnixStream
            // let stream = UnixStream::connect(socket_path).await?;
            //
            // loop {
            //     let ready = stream.ready(Interest::WRITABLE).await?;
            //     println!("{:?}", ready);
            //
            //     if ready.is_writable() {
            //         // Try to write data, this may still fail with `WouldBlock`
            //         // if the readiness event is a false positive.
            //         let n = extract_number_after_double_arrow(line.trim());
            //         // TODO: replace with logic to get wallpaper path
            //         let wallpaper = get_nth_file("/nix/store/xl4p5kciyn2kahc3kpafvgjwqlj0q8yy-khanelinix.wallpapers/share/wallpapers/", n.expect("Expected number"));
            //         let command_str = format!("wallpaper DP-1,{:?}", wallpaper);
            //
            //         match stream.try_write(command_str.as_bytes()) {
            //             Ok(n) => {
            //                 println!("write {} bytes", n);
            //             }
            //             Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            //             Err(e) => {
            //                 return Err(e.into());
            //             }
            //         }
            //     }
            // }
        }
        _ => println!("Unknown event: {}", line),
    }

    return Ok(());
}

fn extract_after_double_arrow(input_string: &str) -> Option<&str> {
    println!("String Extract Input string: {}**", input_string);
    // Split the string by "->" (maximum of one split)
    let parts = input_string.trim().splitn(2, ">>").collect::<Vec<&str>>();
    println!("Parts: {:?}", parts);

    // Check if there's a part after the double arrow
    if parts.len() > 1 {
        println!("Parts length: {}", parts.len());
        return Some(parts[1]);
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

    // Check if there are enough files
    if files.len() < n as usize {
        println!(
            "Error: Not enough files in '{}' to find the {}th file.",
            directory, n
        );
        return None;
    }

    // Get the nth file name
    let file_name = &files[n as usize - 1];

    // Get full path
    let full_path = format!("{}{}", directory, file_name.to_str().unwrap());

    println!("File: {}", full_path);

    Some(full_path)
}
