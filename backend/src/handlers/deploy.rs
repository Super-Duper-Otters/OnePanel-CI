use crate::models::{PushImageReq, Server};
use crate::onepanel::OnePanelClient;
use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

#[utoipa::path(
    post,
    path = "/api/deploy/image",
    request_body = PushImageReq,
    responses(
        (status = 200, description = "Image pushed successfully"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn push_image_to_server_inner(
    db: &crate::db::DbPool,
    server_id: i64,
    image_tag: &str,
) -> Result<(), anyhow::Error> {
    // 1. Get Server
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(server_id)
        .fetch_optional(db)
        .await?;

    let server = server.ok_or_else(|| anyhow::anyhow!("Server not found"))?;

    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let temp_filename = format!("image_{}.tar", timestamp);
    let temp_path = std::env::temp_dir().join(&temp_filename);

    // 2. Docker Save
    // Ensure docker is in path
    println!("Executing docker save -o {:?} {}", temp_path, image_tag);
    let status = tokio::process::Command::new("docker")
        .arg("save")
        .arg("-o")
        .arg(&temp_path)
        .arg(image_tag)
        .status()
        .await?;

    if !status.success() {
        return Err(anyhow::anyhow!("docker save failed"));
    }

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

    // Cleanup local file
    let _ = std::fs::remove_file(&temp_path);

    let remote_path = upload_res?;
    println!("Upload successful, path: {}", remote_path);

    // 4. Load Image
    println!("Loading image from {}", remote_path);
    OnePanelClient::load_image(&server.host, server.port, &server.api_key, &remote_path).await?;

    Ok(())
}

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
    match push_image_to_server_inner(&state.db, payload.server_id, &payload.image_tag).await {
        Ok(_) => (StatusCode::OK, "Image pushed successfully").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
