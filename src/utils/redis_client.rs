use anyhow::{Context, Result};
use redis::Client;

pub fn create_client(url: &str) -> Result<Client> {
    Client::open(url)
        .context("Failed to create Redis client")
}
