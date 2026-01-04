use crate::models::{CreateServerRequest, DashboardResponse, Server, ServerResponse};
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
    path = "/api/servers",
    responses(
        (status = 200, description = "List all servers", body = Vec<ServerResponse>)
    )
)]
pub async fn list_servers(State(state): State<AppState>) -> impl IntoResponse {
    let servers = sqlx::query_as::<_, Server>("SELECT * FROM servers")
        .fetch_all(&*state.db)
        .await
        .unwrap_or(vec![]);

    let responses: Vec<ServerResponse> = servers
        .into_iter()
        .map(|s| ServerResponse {
            id: s.id,
            name: s.name,
            host: s.host,
            port: s.port,
        })
        .collect();

    Json(responses)
}

#[utoipa::path(
    post,
    path = "/api/servers",
    request_body = CreateServerRequest,
    responses(
        (status = 201, description = "Server added"),
        (status = 500, description = "Failed to add server")
    )
)]
pub async fn add_server(
    State(state): State<AppState>,
    Json(payload): Json<CreateServerRequest>,
) -> impl IntoResponse {
    let res = sqlx::query("INSERT INTO servers (name, host, port, api_key) VALUES (?, ?, ?, ?)")
        .bind(payload.name)
        .bind(payload.host)
        .bind(payload.port)
        .bind(payload.api_key)
        .execute(&*state.db)
        .await;

    match res {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/servers/{id}",
    request_body = CreateServerRequest,
    params(
        ("id" = i64, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "Server updated"),
        (status = 500, description = "Failed to update server")
    )
)]
pub async fn update_server(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<CreateServerRequest>,
) -> impl IntoResponse {
    let res =
        sqlx::query("UPDATE servers SET name = ?, host = ?, port = ?, api_key = ? WHERE id = ?")
            .bind(payload.name)
            .bind(payload.host)
            .bind(payload.port)
            .bind(payload.api_key)
            .bind(id)
            .execute(&*state.db)
            .await;

    match res {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/servers/{id}",
    params(
        ("id" = i64, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "Server deleted"),
        (status = 500, description = "Failed to delete server")
    )
)]
pub async fn delete_server(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM servers WHERE id = ?")
        .bind(id)
        .execute(&*state.db)
        .await;

    match res {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/servers/{id}/status",
    params(
        ("id" = i64, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "Get server status", body = DashboardResponse),
        (status = 404, description = "Server not found")
    )
)]
pub async fn get_server_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None);

    if let Some(server) = server {
        match OnePanelClient::get_os_info(&server.host, server.port, &server.api_key).await {
            Ok(info) => (StatusCode::OK, Json(info)).into_response(),
            Err(e) => (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
        }
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
