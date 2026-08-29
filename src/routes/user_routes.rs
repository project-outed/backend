use axum::{
    routing::{get, post, patch},
    Router,
};
use crate::state::AppState;
use crate::handlers::users;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", post(users::create_user).get(users::get_users))
        .route("/users/{id}", get(users::get_user).delete(users::delete_user))
        .route("/users/{id}/providers", get(users::get_user_providers).patch(users::update_providers))
        .route("/users/{id}/mail-verified", patch(users::update_mail_verified))
}
