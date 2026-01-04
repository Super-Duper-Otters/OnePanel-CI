use crate::fs::{self, FileEntry, ListRequest, ScanRequest};
use axum::{http::StatusCode, response::IntoResponse, Json};

#[utoipa::path(
    post,
    path = "/api/fs/list",
    request_body = ListRequest,
    responses(
        (status = 200, description = "List directory contents", body = Vec<FileEntry>)
    )
)]
pub async fn list_directory(Json(payload): Json<ListRequest>) -> impl IntoResponse {
    match fs::list_directory(payload.path) {
        Ok(entries) => (StatusCode::OK, Json(entries)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/fs/scan",
    request_body = ScanRequest,
    responses(
        (status = 200, description = "Scan for git repositories", body = Vec<String>)
    )
)]
pub async fn scan_directory(Json(payload): Json<ScanRequest>) -> impl IntoResponse {
    let repos = fs::scan_for_git_repos(&payload.path);
    (StatusCode::OK, Json(repos)).into_response()
}
