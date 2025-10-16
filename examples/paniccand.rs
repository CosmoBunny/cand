use cand::{Logger, black_box_cand};
use reqwest::Client;
use std::time::Instant;
use tokio;

#[tokio::main]
async fn main() {
    // Set up the emergency panic handler using the default logger.
    black_box_cand!();

    // Initialize the logger.
    let mut logger = Logger(Instant::now(), ());

    logger.log_ok("Starting HTTP requests with reqwest.");

    // Create a blocking HTTP client.
    let client = Client::new();

    // First request: Successful attempt to a valid URL.
    let response = client.get("https://www.example.com").send().await.unwrap();

    println!("{}", response.text().await.unwrap());

    // Last request: Attempt to an invalid URL, which will fail and panic on unwrap.
    let _ = client
        .get("https://invalid-url.example")
        .send()
        .await
        .unwrap();

    // This line will not be reached due to the panic above.
    logger.log_ok("All requests completed.");
}
