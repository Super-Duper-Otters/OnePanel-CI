use crate::models::{CreateNotificationReq, Notification};
use crate::state::AppState;
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/notifications",
    responses(
        (status = 200, description = "List all notifications", body = Vec<Notification>)
    )
)]
pub async fn list_notifications(State(state): State<AppState>) -> impl IntoResponse {
    let notifications: Result<Vec<Notification>, sqlx::Error> = sqlx::query_as::<_, Notification>(
        "SELECT id, type as type_, title, detail, status, timestamp, duration, server_name FROM notifications ORDER BY timestamp DESC"
    )
    .fetch_all(&*state.db)
    .await;

    match notifications {
        Ok(notes) => Json::<Vec<Notification>>(notes).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/notifications",
    request_body = CreateNotificationReq,
    responses(
        (status = 200, description = "Create notification")
    )
)]
pub async fn create_notification(
    State(state): State<AppState>,
    Json(payload): Json<CreateNotificationReq>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let res = sqlx::query(
        "INSERT INTO notifications (id, type, title, detail, status, timestamp, duration, server_name) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(payload.type_)
    .bind(payload.title)
    .bind(payload.detail)
    .bind(payload.status)
    .bind(payload.timestamp)
    .bind(payload.duration)
    .bind(payload.server_name)
    .execute(&*state.db)
    .await;

    match res {
        Ok(_) => axum::http::StatusCode::OK.into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/notifications/clear",
    responses(
        (status = 200, description = "Clear all notifications")
    )
)]
pub async fn clear_notifications(State(state): State<AppState>) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM notifications")
        .execute(&*state.db)
        .await;

    match res {
        Ok(_) => axum::http::StatusCode::OK.into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
