use axum::{response::IntoResponse, Json};

pub async fn get_version() -> impl IntoResponse {
    let version = env!("APP_VERSION");
    Json(serde_json::json!({ "version": version }))
}
