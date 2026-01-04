use crate::git::get_repo_status;
use crate::models::{CreateDirectoryRequest, DirectoryResponse, Repository};
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

#[utoipa::path(
    get,
    path = "/api/directories",
    responses(
        (status = 200, description = "List all repositories", body = Vec<DirectoryResponse>)
    )
)]
pub async fn list_repositories(State(state): State<AppState>) -> impl IntoResponse {
    let repos = sqlx::query_as::<_, Repository>("SELECT * FROM repositories")
        .fetch_all(&*state.db)
        .await
        .unwrap_or(vec![]);

    let mut responses = Vec::new();

    for repo in repos {
        match get_repo_status(&repo.path) {
            Ok(status) => responses.push(DirectoryResponse {
                path: repo.path,
                git_status: Some(status),
                error: None,
            }),
            Err(e) => responses.push(DirectoryResponse {
                path: repo.path,
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
        (status = 201, description = "Repository added"),
        (status = 500, description = "Database error")
    )
)]
pub async fn add_repository(
    State(state): State<AppState>,
    Json(payload): Json<CreateDirectoryRequest>,
) -> impl IntoResponse {
    // Check if it's already there? UNIQUE constraint handles it.
    let res = sqlx::query("INSERT INTO repositories (path) VALUES (?)")
        .bind(payload.path)
        .execute(&*state.db)
        .await;

    match res {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/directories",
    request_body = CreateDirectoryRequest,
    responses(
        (status = 200, description = "Repository removed"),
        (status = 500, description = "Database error")
    )
)]
pub async fn remove_repository(
    State(state): State<AppState>,
    Json(payload): Json<CreateDirectoryRequest>,
) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM repositories WHERE path = ?")
        .bind(payload.path)
        .execute(&*state.db)
        .await;

    match res {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
