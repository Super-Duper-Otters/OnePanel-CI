use crate::docker::{self, ContainerSummary, DockerImage, DockerInfo, PullImageRequest};
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

#[utoipa::path(
    get,
    path = "/api/docker/containers",
    responses(
        (status = 200, description = "List containers", body = Vec<ContainerSummary>),
        (status = 500, description = "Error", body = String)
    )
)]
pub async fn list_containers() -> impl IntoResponse {
    match docker::list_containers().await {
        Ok(containers) => (StatusCode::OK, Json(containers)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/docker/containers/{id}/start",
    params(
        ("id" = String, Path, description = "Container ID")
    ),
    responses(
        (status = 200, description = "Container started"),
        (status = 500, description = "Error", body = String)
    )
)]
pub async fn start_container(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match docker::start_container(&id).await {
        Ok(_) => (StatusCode::OK, "Container started").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/docker/containers/{id}/stop",
    params(
        ("id" = String, Path, description = "Container ID")
    ),
    responses(
        (status = 200, description = "Container stopped"),
        (status = 500, description = "Error", body = String)
    )
)]
pub async fn stop_container(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match docker::stop_container(&id).await {
        Ok(_) => (StatusCode::OK, "Container stopped").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/docker/containers/{id}",
    params(
        ("id" = String, Path, description = "Container ID")
    ),
    responses(
        (status = 200, description = "Container removed"),
        (status = 500, description = "Error", body = String)
    )
)]
pub async fn remove_container(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match docker::remove_container(&id).await {
        Ok(_) => (StatusCode::OK, "Container removed").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/docker/containers/{id}/logs",
    params(
        ("id" = String, Path, description = "Container ID")
    ),
    responses(
        (status = 200, description = "Container logs", body = String),
        (status = 500, description = "Error", body = String)
    )
)]
pub async fn get_container_logs(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match docker::get_container_logs(&id).await {
        Ok(logs) => (StatusCode::OK, logs).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/docker/images",
    responses(
        (status = 200, description = "List images", body = Vec<DockerImage>),
        (status = 500, description = "Error", body = String)
    )
)]
pub async fn list_images() -> impl IntoResponse {
    match docker::list_images().await {
        Ok(images) => (StatusCode::OK, Json(images)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/docker/images/pull",
    request_body = PullImageRequest,
    responses(
        (status = 200, description = "Image pulled"),
        (status = 500, description = "Error", body = String)
    )
)]
pub async fn pull_image(Json(req): Json<docker::PullImageRequest>) -> impl IntoResponse {
    match docker::pull_image(&req.image).await {
        Ok(_) => (StatusCode::OK, "Image pulled").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/docker/images/{id}",
    params(
        ("id" = String, Path, description = "Image ID")
    ),
    responses(
        (status = 200, description = "Image removed"),
        (status = 500, description = "Error", body = String)
    )
)]
pub async fn remove_image(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match docker::remove_image(&id).await {
        Ok(_) => (StatusCode::OK, "Image removed").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/docker/prune",
    responses(
        (status = 200, description = "Images pruned", body = String),
        (status = 500, description = "Error", body = String)
    )
)]
pub async fn prune_images() -> impl IntoResponse {
    match docker::prune_images().await {
        Ok(output) => (StatusCode::OK, output).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}
