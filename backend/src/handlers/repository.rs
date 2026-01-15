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
pub async fn list_repositories_inner(
    db: &crate::db::DbPool,
) -> Result<Vec<DirectoryResponse>, anyhow::Error> {
    let repos = sqlx::query_as::<_, Repository>("SELECT * FROM repositories")
        .fetch_all(db)
        .await?;

    let mut responses = Vec::new();

    for repo in repos {
        match get_repo_status(&repo.path) {
            Ok(status) => responses.push(DirectoryResponse {
                path: repo.path.clone(),
                docker_image_name: repo.docker_image_name.clone(),
                git_status: Some(status),
                error: None,
            }),
            Err(e) => responses.push(DirectoryResponse {
                path: repo.path.clone(),
                docker_image_name: repo.docker_image_name.clone(),
                git_status: None,
                error: Some(e),
            }),
        }
    }

    Ok(responses)
}

#[utoipa::path(
    get,
    path = "/api/directories",
    responses(
        (status = 200, description = "List all repositories", body = Vec<DirectoryResponse>)
    )
)]
pub async fn list_repositories(State(state): State<AppState>) -> impl IntoResponse {
    match list_repositories_inner(&state.db).await {
        Ok(data) => Json(data).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
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

use crate::models::{GetDockerConfigReq, UpdateDockerConfigReq};

#[utoipa::path(
    post,
    path = "/api/directories/config/update",
    request_body = UpdateDockerConfigReq,
    responses(
        (status = 200, description = "Config updated"),
        (status = 500, description = "Database error")
    )
)]
pub async fn update_docker_config(
    State(state): State<AppState>,
    Json(payload): Json<UpdateDockerConfigReq>,
) -> impl IntoResponse {
    // Upsert logic: If path exists, update. If not, insert?
    // Repositories are usually added explicitly. But if user browses a dynamic path, we might want to track it.
    // For now, let's assume we insert-or-update based on path unique constraint.
    let res = sqlx::query(
        "INSERT INTO repositories (path, docker_image_name) VALUES (?, ?) 
         ON CONFLICT(path) DO UPDATE SET docker_image_name = excluded.docker_image_name",
    )
    .bind(&payload.path)
    .bind(&payload.docker_image_name)
    .execute(&*state.db)
    .await;

    match res {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/directories/config/get",
    request_body = GetDockerConfigReq,
    responses(
        (status = 200, description = "Get config", body = DirectoryResponse) 
    )
)]
pub async fn get_docker_config(
    State(state): State<AppState>,
    Json(payload): Json<GetDockerConfigReq>,
) -> impl IntoResponse {
    let repo = sqlx::query_as::<_, Repository>("SELECT * FROM repositories WHERE path = ?")
        .bind(&payload.path)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None);

    match repo {
        Some(r) => Json(DirectoryResponse {
            path: r.path,
            docker_image_name: r.docker_image_name,
            git_status: None,
            error: None,
        })
        .into_response(),
        None => Json(DirectoryResponse {
            path: payload.path,
            docker_image_name: None,
            git_status: None,
            error: None,
        })
        .into_response(),
    }
}
