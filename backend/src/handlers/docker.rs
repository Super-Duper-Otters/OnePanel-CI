use crate::docker::{self, DockerImage, DockerInfo};
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

#[derive(serde::Deserialize)]
pub struct ListTagsRequest {
    pub image: String,
}

#[utoipa::path(
    get,
    path = "/api/docker/tags",
    params(
        ("image" = String, Query, description = "Image name to filter tags")
    ),
    responses(
        (status = 200, description = "List of images", body = Vec<DockerImage>),
        (status = 500, description = "Error", body = String)
    )
)]
pub async fn list_tags(
    axum::extract::Query(params): axum::extract::Query<ListTagsRequest>,
) -> impl IntoResponse {
    match docker::list_tags(&params.image).await {
        Ok(images) => (StatusCode::OK, Json(images)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/docker/build",
    request_body = docker::DockerBuildRequest,
    responses(
        (status = 200, description = "Build Output", body = String),
        (status = 500, description = "Build Failed", body = String)
    )
)]
pub async fn build_image(Json(req): Json<docker::DockerBuildRequest>) -> impl IntoResponse {
    match docker::build_image(req).await {
        Ok(output) => (StatusCode::OK, output).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}
