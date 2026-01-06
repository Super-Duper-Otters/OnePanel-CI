use crate::models::Server;
use crate::onepanel::OnePanelClient;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ImageDeploymentsQuery {
    /// Base image name to search for (e.g., "my-app" without tag)
    pub image_base: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ImageDeployment {
    pub server_id: i64,
    pub server_name: String,
    pub compose_name: String,
    pub image_tag: String,
}

#[utoipa::path(
    get,
    path = "/api/image-deployments",
    tag = "Deploy",
    params(ImageDeploymentsQuery),
    responses(
        (status = 200, description = "List of deployments for the image", body = Vec<ImageDeployment>)
    )
)]
pub async fn get_image_deployments(
    State(state): State<AppState>,
    Query(query): Query<ImageDeploymentsQuery>,
) -> impl IntoResponse {
    let image_base = query.image_base;

    // Get all servers
    let servers = match sqlx::query_as::<_, Server>("SELECT * FROM servers")
        .fetch_all(&*state.db)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let mut deployments: Vec<ImageDeployment> = Vec::new();

    // Build regex to match image references
    // Matches patterns like: image: my-app:1.0.0 or image: "my-app:latest"
    let pattern = format!(
        r#"image:\s*["']?{}(:[^\s"']+)?["']?"#,
        regex::escape(&image_base)
    );
    let re = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invalid regex: {}", e),
            )
                .into_response();
        }
    };

    for server in servers {
        // Get composes for this server
        let composes =
            match OnePanelClient::list_composes(&server.host, server.port, &server.api_key).await {
                Ok(c) => c,
                Err(_) => continue, // Skip server if we can't get composes
            };

        for compose in composes {
            let compose_name = compose
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let compose_path = compose.get("path").and_then(|v| v.as_str()).unwrap_or("");

            if compose_path.is_empty() {
                continue;
            }

            // Read compose file content
            let content = match OnePanelClient::read_file(
                &server.host,
                server.port,
                &server.api_key,
                compose_path,
            )
            .await
            {
                Ok(c) => c,
                Err(_) => continue, // Skip if we can't read the file
            };

            // Find all image references matching our base name
            for cap in re.captures_iter(&content) {
                let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");
                // Extract the image tag from the match
                let image_tag = full_match
                    .trim_start_matches("image:")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();

                deployments.push(ImageDeployment {
                    server_id: server.id,
                    server_name: server.name.clone(),
                    compose_name: compose_name.clone(),
                    image_tag,
                });
            }
        }
    }

    (StatusCode::OK, Json(deployments)).into_response()
}
