use axum::{
    extract::State, 
    Json, 
    http::{StatusCode, HeaderMap},
    body::Bytes,
};
use crate::state::AppState;
use crate::models::DiscordEventWrapper;
use tracing::{error, info, debug};
use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use std::env;

use super::authorized::handle_authorized;
use super::deauthorized::handle_deauthorized;

pub async fn handle_discord_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let signature = headers
        .get("X-Signature-Ed25519")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let timestamp = headers
        .get("X-Signature-Timestamp")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let public_key_hex = env::var("DISCORD_PUBLIC_KEY").map_err(|_| {
        error!("DISCORD_PUBLIC_KEY not set in environment");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !verify_discord_signature(&public_key_hex, signature, timestamp, &body) {
        error!("Invalid Discord signature");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let json_value: serde_json::Value = serde_json::from_slice(&body).map_err(|e| {
        error!("Failed to parse raw body as JSON: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    info!("Full Payload: {}", json_value);

    let payload: DiscordEventWrapper = match serde_json::from_value(json_value.clone()) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to map payload to DiscordEventWrapper: {}. Body: {}", e, json_value);
            return Ok(Json(serde_json::json!({ "type": 1 })));
        }
    };

    let interaction_type = payload.event_type;
    debug!("Received Discord interaction type: {}", interaction_type);

    if let Some(event) = &payload.event {
        match event.type_name.as_str() {
            "APPLICATION_AUTHORIZED" => {
                info!("Handling APPLICATION_AUTHORIZED event");
                handle_authorized(&state, &event.data).await?;
            }
            "APPLICATION_DEAUTHORIZED" => {
                info!("Handling APPLICATION_DEAUTHORIZED event");
                handle_deauthorized(&state, &event.data).await?;
            }
            _ => {
                info!("Unhandled Discord event type: {}", event.type_name);
            }
        }
    }

    Ok(Json(serde_json::json!({ "type": 1 })))
}

fn verify_discord_signature(
    public_key_hex: &str,
    signature_hex: &str,
    timestamp: &str,
    body: &[u8],
) -> bool {
    let public_key_bytes = match hex::decode(public_key_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let signature_bytes = match hex::decode(signature_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let public_key = match VerifyingKey::from_bytes(public_key_bytes[..32].try_into().unwrap_or(&[0; 32])) {
        Ok(k) => k,
        Err(_) => return false,
    };

    let signature = match Signature::from_slice(&signature_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let mut message = timestamp.as_bytes().to_vec();
    message.extend_from_slice(body);

    public_key.verify(&message, &signature).is_ok()
}
