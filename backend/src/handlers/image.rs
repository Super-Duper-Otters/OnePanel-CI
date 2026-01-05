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
    path = "/api/servers/{id}/images",
    tag = "Image",
    params(
        ("id" = i64, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "List images", body = Vec<serde_json::Value>)
    )
)]
pub async fn list_images(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None);

    if let Some(s) = server {
        match OnePanelClient::list_images(&s.host, s.port, &s.api_key).await {
            Ok(list) => return (StatusCode::OK, Json(list)).into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
    (StatusCode::NOT_FOUND, "Server not found").into_response()
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct RemoveImageReq {
    pub id: String,
    pub force: bool,
}

#[utoipa::path(
    post,
    path = "/api/servers/{id}/images/remove",
    tag = "Image",
    request_body = RemoveImageReq,
    params(
        ("id" = i64, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "Image removed")
    )
)]
pub async fn remove_image(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<RemoveImageReq>,
) -> impl IntoResponse {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None);

    if let Some(s) = server {
        match OnePanelClient::remove_image(&s.host, s.port, &s.api_key, &payload.id, payload.force)
            .await
        {
            Ok(_) => return (StatusCode::OK, "Image removed").into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
    (StatusCode::NOT_FOUND, "Server not found").into_response()
}
