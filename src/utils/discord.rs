use reqwest::Client;
use serde_json::Value;
use std::env;
use anyhow::Result;

pub async fn get_discord_info(client: &Client, discord_id: &str) -> Result<Option<Value>> {
    let token = env::var("BOT_TOKEN").map_err(|_| anyhow::anyhow!("BOT_TOKEN not set"))?;
    
    let url = format!("https://discord.com/api/v10/users/{}", discord_id);
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bot {}", token))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Ok(None);
    }
    
    let response: Value = resp.json().await?;
    
    let username = response["username"].as_str().unwrap_or("Unknown").to_string();
    let discriminator = response["discriminator"].as_str().unwrap_or("0000");
    let full_username = if discriminator == "0" { username } else { format!("{}#{}", username, discriminator) };

    let avatar = if let Some(avatar_hash) = response["avatar"].as_str() {
        let extension = if avatar_hash.starts_with("a_") { "gif" } else { "png" };
        Some(format!(
            "https://cdn.discordapp.com/avatars/{}/{}.{}",
            discord_id, avatar_hash, extension
        ))
    } else {
        None
    };

    Ok(Some(serde_json::json!({
        "username": full_username,
        "id": discord_id,
        "avatar": avatar
    })))
}
