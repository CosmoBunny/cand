use std::{error::Error, io, time::Instant};

use cand::{Logger, StorageProvider, TimeProvider};

fn main() -> Result<(), Box<dyn Error>> {
    let mut logger = Logger(Instant::now(), ());

    // Get file path from command-line arguments or prompt user
    let args: Vec<String> = std::env::args().collect();
    let file_path = if args.len() > 1 {
        args[1].clone()
    } else {
        logger.log_info("No file path provided as argument, please enter a file path:");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    // Log attempt to read file
    logger.log_info(format_args!("Attempting to read file: {}", file_path));

    // Read file content
    let (content, logger) = logger.try_get(try_read_file(&file_path), file_read_fail);

    // Convert to string
    let (str_content, mut logger) = logger.try_get(try_convert_string(content), |mut logger| {
        logger.log_warn("Failed to convert file content to UTF-8 string");
        logger.log_ok("Safety Protocol: Closing Application");
    });

    // Log success and print content
    logger.log_ok("File read successfully");
    println!("File content:\n{}", str_content);

    Ok(())
}

fn try_read_file(path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let content = std::fs::read(path)?;
    Ok(content)
}

fn try_convert_string(value: Vec<u8>) -> Result<String, Box<dyn Error>> {
    let str_content = String::from_utf8(value)?;
    Ok(str_content)
}

fn file_read_fail<T: TimeProvider, S: StorageProvider>(mut logger: Logger<T, S>) {
    logger.log_err("Unable to read file");
    logger.log_ok("Safety Protocol: Terminating");
}
