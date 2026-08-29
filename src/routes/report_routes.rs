use axum::{routing::get, routing::post, routing::put, Router};
use crate::state::AppState;
use crate::handlers::reports;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/reports", post(reports::create_report))
        .route("/reports/lookup", get(reports::lookup_reports))
        .route("/reports/upload", post(reports::upload_evidence))
        .route("/reports/{id}/accept", post(reports::accept_report))
        .route("/reports/{id}/decline", post(reports::decline_report))
        .route("/reports/{id}", put(reports::save_report))
        .route("/reports/{id}/evidence", get(reports::get_evidence))
        .route("/reports/{id}/evidence/{evidence_id}", get(reports::get_evidence_file))
}
