use crate::git::{get_repo_status, GitStatus};
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod fs;
pub mod git;

#[derive(Deserialize, ToSchema)]
pub struct CreateDirectoryRequest {
    pub path: String,
}

#[derive(Serialize, ToSchema)]
pub struct DirectoryResponse {
    pub path: String,
    pub git_status: Option<GitStatus>,
    pub error: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/directories",
    responses(
        (status = 200, description = "List all directories", body = Vec<DirectoryResponse>)
    )
)]
pub async fn list_directories(State(state): State<AppState>) -> impl IntoResponse {
    let dirs = state.directories.lock().unwrap();
    let mut responses = Vec::new();

    for path in dirs.iter() {
        match get_repo_status(path) {
            Ok(status) => responses.push(DirectoryResponse {
                path: path.clone(),
                git_status: Some(status),
                error: None,
            }),
            Err(e) => responses.push(DirectoryResponse {
                path: path.clone(),
                git_status: None,
                error: Some(e),
            }),
        }
    }

    Json(responses)
}

#[utoipa::path(
    post,
    path = "/api/directories",
    request_body = CreateDirectoryRequest,
    responses(
        (status = 201, description = "Directory added"),
        (status = 400, description = "Invalid path")
    )
)]
pub async fn add_directory(
    State(state): State<AppState>,
    Json(payload): Json<CreateDirectoryRequest>,
) -> impl IntoResponse {
    let mut dirs = state.directories.lock().unwrap();
    // In future: Verify it's a valid path/git repo?
    dirs.insert(payload.path);
    StatusCode::CREATED
}

#[utoipa::path(
    delete,
    path = "/api/directories",
    request_body = CreateDirectoryRequest,
    responses(
        (status = 200, description = "Directory removed"),
    )
)]
pub async fn remove_directory(
    State(state): State<AppState>,
    Json(payload): Json<CreateDirectoryRequest>,
) -> impl IntoResponse {
    let mut dirs = state.directories.lock().unwrap();
    dirs.remove(&payload.path);
    StatusCode::OK
}
