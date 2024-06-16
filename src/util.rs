use log::debug;
use log::error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::from_utf8;

pub fn extract_after_double_arrow(input_string: &str) -> Option<String> {
    debug!("String Extract Input string: {}**", input_string);
    // Split the input string by "\n"
    let parts: Vec<&str> = input_string.split('\n').collect();

    // Extract the first part before the "\n" character
    if let Some(first_part) = parts.first() {
        // Trim any extra spaces
        let cleaned_string = first_part.trim();

        // Split the cleaned string by ">>" (maximum of one split)
        let parts: Vec<&str> = cleaned_string.splitn(2, ">>").collect();
        debug!("Parts: {:?}", parts);

        // Check if there's a part after the double arrow
        if parts.len() > 1 {
            debug!("Parts length: {}", parts.len());
            return Some(parts[1].to_string());
        }
    }

    // No part after double arrow, return None
    None
}

pub fn extract_number_after_double_arrow(input_string: &str) -> Option<u32> {
    // Extract the part after the double arrow
    debug!("Number Extract Input string: {}**", input_string);
    let decoded_string = from_utf8(input_string.as_bytes()).unwrap_or(input_string);
    let trimmed_string = decoded_string.trim_end_matches(char::from(0));

    debug!(
        "Number Extract Input string (decoded): {}**",
        decoded_string
    );

    if let Some(part_after_arrow) = extract_after_double_arrow(trimmed_string) {
        debug!("Part after arrow: {}", part_after_arrow);

        // Parse the extracted part as a u32
        match part_after_arrow.trim().parse::<u32>() {
            Ok(number) => {
                debug!("Number: {}", number);
                return Some(number);
            }
            Err(_) => {
                // Handle the error if the part is not a valid number
                error!("Error: Invalid number format: {}", part_after_arrow);
            }
        }
    }

    None // No number found
}

pub fn get_nth_file(directory: &str, n: u32) -> Result<PathBuf, String> {
    debug!("Directory: {}", directory);

    // Validate path early
    let path = Path::new(directory);
    if !path.exists() {
        return Err(format!(
            "Error: Directory '{}' does not exist.",
            path.display()
        ));
    }

    // Read directory contents
    let mut files: Vec<_> = fs::read_dir(path)
        .map_err(|err| format!("Error reading directory: {}", err))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect();

    // error if no files
    if files.is_empty() {
        return Err(format!("Directory '{}' contains no files.", path.display()));
    }

    files.sort();

    // Handle potential index out of bounds
    let index = std::cmp::min(n as usize - 1, files.len() - 1);

    let file_path = files.get(index).ok_or_else(|| {
        format!(
            "Directory '{}' contains fewer than {} files.",
            path.display(),
            n
        )
    })?;

    debug!("File: {}", file_path.display());

    Ok(file_path.to_path_buf())
}
