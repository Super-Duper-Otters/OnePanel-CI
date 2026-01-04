use crate::git::{self, CommitInfo, FileStatus};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct GitLogRequest {
    pub path: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize, ToSchema)]
pub struct GitStatusRequest {
    pub path: String,
}

#[utoipa::path(
    post,
    path = "/api/git/log",
    request_body = GitLogRequest,
    responses(
        (status = 200, description = "Get git log", body = Vec<CommitInfo>)
    )
)]
pub async fn get_git_log(Json(payload): Json<GitLogRequest>) -> impl IntoResponse {
    let limit = payload.limit.unwrap_or(10);
    match git::get_commit_log(&payload.path, limit) {
        Ok(commits) => (StatusCode::OK, Json(commits)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/git/status",
    request_body = GitStatusRequest,
    responses(
        (status = 200, description = "Get git status", body = Vec<FileStatus>)
    )
)]
pub async fn get_git_status(Json(payload): Json<GitStatusRequest>) -> impl IntoResponse {
    match git::get_detailed_status(&payload.path) {
        Ok(statuses) => (StatusCode::OK, Json(statuses)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}
