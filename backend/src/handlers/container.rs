use crate::models::{ContainerOperationReq, Server};
use crate::onepanel::OnePanelClient;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

#[utoipa::path(
    get,
    path = "/api/servers/{id}/containers",
    tag = "Container",
    params(
        ("id" = i64, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "List containers", body = Vec<serde_json::Value>)
    )
)]
pub async fn list_containers(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None);

    if let Some(s) = server {
        match OnePanelClient::list_containers(&s.host, s.port, &s.api_key).await {
            Ok(list) => return (StatusCode::OK, Json(list)).into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
    (StatusCode::NOT_FOUND, "Server not found").into_response()
}

#[utoipa::path(
    post,
    path = "/api/servers/{id}/containers/operate",
    tag = "Container",
    request_body = ContainerOperationReq,
    params(
        ("id" = i64, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "Operation successful")
    )
)]
pub async fn operate_container(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<ContainerOperationReq>,
) -> impl IntoResponse {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None);

    if let Some(s) = server {
        match OnePanelClient::operate_container(
            &s.host,
            s.port,
            &s.api_key,
            payload.names,
            payload.operation,
        )
        .await
        {
            Ok(_) => return StatusCode::OK.into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
    (StatusCode::NOT_FOUND, "Server not found").into_response()
}

#[derive(Deserialize)]
pub struct LogsQuery {
    container: String,
}

#[utoipa::path(
    get,
    path = "/api/servers/{id}/containers/logs",
    tag = "Container",
    params(
        ("id" = i64, Path, description = "Server ID"),
        ("container" = String, Query, description = "Container name/id")
    ),
    responses(
        (status = 200, description = "Container logs", body = String)
    )
)]
pub async fn get_logs(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<LogsQuery>,
) -> impl IntoResponse {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None);

    if let Some(s) = server {
        match OnePanelClient::get_container_logs(&s.host, s.port, &s.api_key, query.container).await
        {
            Ok(logs) => return (StatusCode::OK, logs).into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
    (StatusCode::NOT_FOUND, "Server not found").into_response()
}
