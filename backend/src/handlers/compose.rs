use crate::models::Server;
use crate::onepanel::OnePanelClient;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

#[utoipa::path(
    get,
    path = "/api/servers/{id}/composes",
    tag = "Compose",
    params(
        ("id" = i64, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "List composes", body = Vec<serde_json::Value>)
    )
)]
pub async fn list_composes(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None);

    if let Some(s) = server {
        match OnePanelClient::list_composes(&s.host, s.port, &s.api_key).await {
            Ok(list) => return (StatusCode::OK, Json(list)).into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
    (StatusCode::NOT_FOUND, "Server not found").into_response()
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct GetContentReq {
    pub path: String,
}

#[utoipa::path(
    post,
    path = "/api/servers/{id}/composes/content",
    tag = "Compose",
    request_body = GetContentReq,
    params(
        ("id" = i64, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "Compose content", body = String)
    )
)]
pub async fn get_content(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<GetContentReq>,
) -> impl IntoResponse {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None);

    if let Some(s) = server {
        match OnePanelClient::read_file(&s.host, s.port, &s.api_key, &payload.path).await {
            Ok(content) => return (StatusCode::OK, content).into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
    (StatusCode::NOT_FOUND, "Server not found").into_response()
}
