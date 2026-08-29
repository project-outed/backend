use axum::{extract::{Query, State}, response::Redirect};
use serde::Deserialize;
use serde_json::Value;
use std::env;
use tracing::{error, info, warn};

use crate::state::AppState;

const REDIRECT_SUCCESS: &str = "https://outed.dev/discord/verified-project-outed";
const REDIRECT_ERROR_BASE: &str = "https://outed.dev/verification/error?message=";
const DISCORD_REDIRECT_URI: &str = "https://backend.outed.dev/callback/discord";
const DISCORD_API_BASE: &str = "https://discord.com/api/v10";

#[derive(Debug, Deserialize)]
pub struct DiscordCallbackQuery {
    code: Option<String>,
}

pub async fn discord_callback(
    State(state): State<AppState>,
    Query(params): Query<DiscordCallbackQuery>,
) -> Redirect {
    let code = match params.code.as_deref() {
        Some(code) if !code.trim().is_empty() => code.trim(),
        _ => {
            warn!(?params, "Missing authorization code in callback request");
            return redirect_error("Missing authorization code");
        }
    };

    let client_id = env::var("BOT_CLIENT").unwrap_or_default();
    let client_secret = env::var("BOT_SECRET").unwrap_or_default();
    if client_id.is_empty() || client_secret.is_empty() {
        error!("Discord OAuth client credentials missing");
        return redirect_error("Discord OAuth client credentials are not configured");
    }

    info!("Requesting Discord OAuth2 token");
    let body = format!(
        "client_id={}&client_secret={}&grant_type=authorization_code&code={}&redirect_uri={}",
        url_encode(&client_id),
        url_encode(&client_secret),
        url_encode(code),
        url_encode(DISCORD_REDIRECT_URI),
    );

    let token_response = state.http_client
        .post("https://discord.com/api/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await;

    let token_response = match token_response {
        Ok(resp) => resp,
        Err(err) => {
            error!(?err, "Discord token exchange failed");
            return redirect_error("Communication error occurred");
        }
    };

    let token_status = token_response.status();
    let token_data: Value = match token_response.json().await {
        Ok(value) => value,
        Err(err) => {
            error!(?err, "Discord token response parse failed");
            return redirect_error("Failed to read Discord token response");
        }
    };

    if !token_status.is_success() {
        let message = token_data
            .get("error_description")
            .and_then(Value::as_str)
            .or_else(|| token_data.get("error").and_then(Value::as_str))
            .unwrap_or("Unknown error");
        return redirect_error(message);
    }

    let access_token = token_data
        .get("access_token")
        .and_then(Value::as_str)
        .unwrap_or("");

    if access_token.is_empty() {
        error!(?token_data, "Discord access token missing in token response");
        return redirect_error("Failed to retrieve access token");
    }

    info!("Fetching Discord user info");
    let user_response = state.http_client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(access_token)
        .send()
        .await;

    let user_response = match user_response {
        Ok(resp) => resp,
        Err(err) => {
            error!(?err, "Discord user info request failed");
            return redirect_error("Communication error occurred");
        }
    };

    let user_status = user_response.status();
    let user_data: Value = match user_response.json().await {
        Ok(value) => value,
        Err(err) => {
            error!(?err, "Discord user response parse failed");
            return redirect_error("Failed to read Discord user info");
        }
    };

    if !user_status.is_success() {
        let message = user_data
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Discord API returned an error");
        return redirect_error(message);
    }

    let discord_id = user_data
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("");

    if discord_id.is_empty() {
        error!(?user_data, "Discord user ID missing");
        return redirect_error("Discord user ID is missing");
    }

    let email = user_data
        .get("email")
        .and_then(Value::as_str)
        .map(String::from);

    let guild_id = env::var("GUILD_MAIN").unwrap_or_default();
    let role_id = env::var("DISCORD_ROLE_VERIFIED").unwrap_or_default();
    let bot_token = env::var("BOT_TOKEN").unwrap_or_default();

    if guild_id.is_empty() || role_id.is_empty() || bot_token.is_empty() {
        return redirect_error("Discord guild role configuration is not complete");
    }

    let role_url = format!(
        "{}/guilds/{}/members/{}/roles/{}",
        DISCORD_API_BASE, guild_id, discord_id, role_id
    );

    let role_response = state.http_client
        .put(&role_url)
        .header("Authorization", format!("Bot {}", bot_token))
        .header("Content-Type", "application/json")
        .send()
        .await;

    let role_response = match role_response {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Discord role assignment failed: {:?}", err);
            return redirect_error("Communication error occurred");
        }
    };

    if !role_response.status().is_success() {
        let role_status = role_response.status();
        let body = role_response.text().await.unwrap_or_default();
        error!(status = ?role_status, body = %body, "Discord role assignment failed");
        return redirect_error(&format!("Failed to add role: {}", body));
    }

    let username = user_data.get("username").and_then(Value::as_str).unwrap_or("Unknown");
    let avatar_hash = user_data.get("avatar").and_then(Value::as_str).unwrap_or("");
    let avatar_url = if !avatar_hash.is_empty() {
        format!("https://cdn.discordapp.com/avatars/{}/{}.png", discord_id, avatar_hash)
    } else {
        "".to_string()
    };

    let discord_id_i64: i64 = discord_id.parse().unwrap_or(0);
    let providers_json = serde_json::json!({
        "discord": {
            "id": discord_id,
            "username": username,
            "avatar": avatar_url
        }
    });

    let db_result = sqlx::query(
        "INSERT INTO users (user_id, mail, verified, providers, updated_at)
         VALUES ($1, $2, 1, $3, CURRENT_TIMESTAMP)
         ON CONFLICT (user_id)
         DO UPDATE SET
             mail = COALESCE($2, users.mail),
             verified = 1,
             providers = users.providers || EXCLUDED.providers,
             updated_at = CURRENT_TIMESTAMP"
    )
    .bind(discord_id_i64)
    .bind(email.as_deref())
    .bind(providers_json)
    .execute(&state.db)
    .await;

    match db_result {
        Ok(_) => {}
        Err(err) => error!(?err, "Failed to update local user record"),
    }

    Redirect::temporary(REDIRECT_SUCCESS)
}

fn url_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            b' ' => "+".to_string(),
            _ => format!("%{:02X}", byte),
        })
        .collect()
}

fn redirect_error(message: &str) -> Redirect {
    let encoded = url_encode(message);
    Redirect::temporary(&format!("{}{}", REDIRECT_ERROR_BASE, encoded))
}
