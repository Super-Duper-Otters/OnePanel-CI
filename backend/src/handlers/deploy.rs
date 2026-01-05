use crate::models::{PushImageReq, Server};
use crate::onepanel::OnePanelClient;
use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::process::Command;

#[utoipa::path(
    post,
    path = "/api/deploy/image",
    request_body = PushImageReq,
    responses(
        (status = 200, description = "Image pushed successfully"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn push_image_to_server(
    State(state): State<AppState>,
    Json(payload): Json<PushImageReq>,
) -> impl IntoResponse {
    // 1. Get Server
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(payload.server_id)
        .fetch_optional(&*state.db)
        .await;

    let server = match server {
        Ok(Some(s)) => s,
        Ok(None) => return (StatusCode::NOT_FOUND, "Server not found").into_response(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB Error: {}", e),
            )
                .into_response()
        }
    };

    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let temp_filename = format!("image_{}.tar", timestamp);
    let temp_path = std::env::temp_dir().join(&temp_filename);

    // 2. Docker Save
    // Ensure docker is in path
    println!(
        "Executing docker save -o {:?} {}",
        temp_path, payload.image_tag
    );
    let status = Command::new("docker")
        .arg("save")
        .arg("-o")
        .arg(&temp_path)
        .arg(&payload.image_tag)
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("docker save failed with exit code: {:?}", s.code()),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to execute docker save: {}", e),
            )
                .into_response()
        }
    };

    // 3. Upload File
    println!("Uploading file to server...");
    let remote_dir = "/opt/1panel/tmp"; // Default temp dir
    let upload_res = OnePanelClient::upload_file(
        &server.host,
        server.port,
        &server.api_key,
        &temp_path,
        remote_dir,
    )
    .await;

    let remote_path = match upload_res {
        Ok(p) => {
            println!("Upload successful, path: {}", p);
            p
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Upload failed: {}", e),
            )
                .into_response();
        }
    };

    // 4. Load Image
    println!("Loading image from {}", remote_path);
    let load_res =
        OnePanelClient::load_image(&server.host, server.port, &server.api_key, &remote_path).await;

    // 5. Cleanup local file
    let _ = std::fs::remove_file(&temp_path);

    if let Err(e) = load_res {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Load image failed: {}", e),
        )
            .into_response();
    }

    (StatusCode::OK, "Image pushed successfully").into_response()
}
