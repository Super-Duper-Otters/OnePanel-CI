use crate::docker::{self, DockerInfo};
use axum::{response::IntoResponse, Json};
use reqwest::StatusCode;

#[utoipa::path(
    get,
    path = "/api/docker/info",
    responses(
        (status = 200, description = "Get Docker Info", body = DockerInfo),
        (status = 500, description = "Docker connection failed", body = String)
    )
)]
pub async fn get_info() -> impl IntoResponse {
    match docker::get_docker_info().await {
        Ok(info) => (StatusCode::OK, Json(info)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(), // 500 if docker is down
    }
}
