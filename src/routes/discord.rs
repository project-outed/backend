use axum::{
    Router,
    routing::post,
    extract::State,
    http::StatusCode,
    Json,
};
use crate::state::AppState;
use crate::models::DiscordEventWrapper;
use crate::handlers::discord;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/discord/event", post(discord_event_handler))
}

async fn discord_event_handler(
    State(state): State<AppState>,
    Json(payload): Json<DiscordEventWrapper>,
) -> StatusCode {
    // Discord sends a URL verification ping with type 0 (not an event)
    if payload.event_type == 0 {
        return StatusCode::OK;
    }

    if let Some(event) = &payload.event {
        match event.type_name.as_str() {
            "APPLICATION_AUTHORIZED" => {
                match discord::handle_authorized(&state, &event.data).await {
                    Ok(()) => StatusCode::OK,
                    Err(status) => status,
                }
            }
            _ => {
                tracing::warn!("Unhandled Discord event type: {}", event.type_name);
                StatusCode::OK
            }
        }
    } else {
        tracing::warn!("Received Discord event wrapper with no event body");
        StatusCode::BAD_REQUEST
    }
}
