use axum::http::StatusCode;
use crate::state::AppState;
use tracing::{error, info, debug};

use crate::models::{DiscordEventData, Providers, DiscordProviders};

