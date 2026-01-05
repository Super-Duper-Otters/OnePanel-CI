use crate::models::{CreateServerRequest, DashboardResponse, Server, ServerResponse};
use crate::onepanel::OnePanelClient;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use regex::Regex;

#[utoipa::path(
    get,
    path = "/api/servers",
    responses(
        (status = 200, description = "List all servers", body = Vec<ServerResponse>)
    )
)]
pub async fn list_servers(State(state): State<AppState>) -> impl IntoResponse {
    let servers = sqlx::query_as::<_, Server>("SELECT * FROM servers")
        .fetch_all(&*state.db)
        .await
        .unwrap_or(vec![]);

    let responses: Vec<ServerResponse> = servers
        .into_iter()
        .map(|s| ServerResponse {
            id: s.id,
            name: s.name,
            host: s.host,
            port: s.port,
        })
        .collect();

    Json(responses)
}

#[utoipa::path(
    post,
    path = "/api/servers",
    request_body = CreateServerRequest,
    responses(
        (status = 201, description = "Server added"),
        (status = 500, description = "Failed to add server")
    )
)]
pub async fn add_server(
    State(state): State<AppState>,
    Json(payload): Json<CreateServerRequest>,
) -> impl IntoResponse {
    let res = sqlx::query("INSERT INTO servers (name, host, port, api_key) VALUES (?, ?, ?, ?)")
        .bind(payload.name)
        .bind(payload.host)
        .bind(payload.port)
        .bind(payload.api_key)
        .execute(&*state.db)
        .await;

    match res {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/servers/{id}",
    request_body = CreateServerRequest,
    params(
        ("id" = i64, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "Server updated"),
        (status = 500, description = "Failed to update server")
    )
)]
pub async fn update_server(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<CreateServerRequest>,
) -> impl IntoResponse {
    let res =
        sqlx::query("UPDATE servers SET name = ?, host = ?, port = ?, api_key = ? WHERE id = ?")
            .bind(payload.name)
            .bind(payload.host)
            .bind(payload.port)
            .bind(payload.api_key)
            .bind(id)
            .execute(&*state.db)
            .await;

    match res {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/servers/{id}",
    params(
        ("id" = i64, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "Server deleted"),
        (status = 500, description = "Failed to delete server")
    )
)]
pub async fn delete_server(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM servers WHERE id = ?")
        .bind(id)
        .execute(&*state.db)
        .await;

    match res {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/servers/{id}",
    params(
        ("id" = i64, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "Get server details", body = ServerResponse),
        (status = 404, description = "Server not found")
    )
)]
pub async fn get_server(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None);

    if let Some(s) = server {
        let response = ServerResponse {
            id: s.id,
            name: s.name,
            host: s.host,
            port: s.port,
        };
        Json(response).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

#[utoipa::path(
    get,
    path = "/api/servers/{id}/status",
    params(
        ("id" = i64, Path, description = "Server ID")
    ),
    responses(
        (status = 200, description = "Get server status", body = DashboardResponse),
        (status = 404, description = "Server not found")
    )
)]
pub async fn get_server_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None);

    if let Some(server) = server {
        match OnePanelClient::get_os_info(&server.host, server.port, &server.api_key).await {
            Ok(info) => (StatusCode::OK, Json(info)).into_response(),
            Err(e) => (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
        }
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn proxy_request(
    State(state): State<AppState>,
    Path((id, path)): Path<(i64, String)>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None);

    if let Some(server) = server {
        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let timestamp = chrono::Utc::now().timestamp();
        let api_key = server.api_key.trim();
        let token_raw = format!("1panel{}{}", api_key, timestamp);
        let token_digest = md5::compute(token_raw.as_bytes());
        let token_str = format!("{:x}", token_digest);

        let target_url = format!("http://{}:{}/{}", server.host, server.port, path);
        println!("Proxying to: {}", target_url);

        let method = req.method().clone();
        let headers = req.headers().clone();

        let mut builder = client
            .request(method, &target_url)
            .header("1Panel-Token", token_str)
            .header("1Panel-Timestamp", timestamp.to_string());

        // Forward headers (exclude host to avoid issues)
        for (key, value) in headers.iter() {
            if key.as_str().to_lowercase() != "host" {
                builder = builder.header(key, value);
            }
        }

        // Handle body if present? For GET/Swagger usually not needed but good for completeness.
        // For now, simple GET/proxy for swagger docs.
        // Handling body in a generic proxy is more complex with Axum body types.
        // Assuming primarily GET for Swagger UI.

        match builder.send().await {
            Ok(res) => {
                let status = res.status();
                let headers = res.headers().clone();
                let body = res.bytes().await.unwrap_or_default();

                let mut response_builder = axum::response::Response::builder().status(status);
                for (key, value) in headers.iter() {
                    let key_str = key.as_str().to_lowercase();
                    if key_str != "content-length"
                        && key_str != "transfer-encoding"
                        && key_str != "content-encoding"
                        && key_str != "connection"
                    {
                        response_builder = response_builder.header(key, value);
                    }
                }
                response_builder
                    .body(axum::body::Body::from(body))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
            Err(e) => (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
        }
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn serve_scalar_docs(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None);

    if let Some(server) = server {
        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let timestamp = chrono::Utc::now().timestamp();
        let api_key = server.api_key.trim();
        let token_raw = format!("1panel{}{}", api_key, timestamp);
        let token_digest = md5::compute(token_raw.as_bytes());
        let token_str = format!("{:x}", token_digest);

        let target_url = format!(
            "http://{}:{}/1panel/swagger/doc.json",
            server.host, server.port
        );

        let response = client
            .get(&target_url)
            .header("1Panel-Token", token_str)
            .header("1Panel-Timestamp", timestamp.to_string())
            .send()
            .await;

        let spec_content = match response {
            Ok(res) => res.text().await.unwrap_or_default(),
            Err(_) => "{}".to_string(),
        };
        let html_re = Regex::new(r"<!DOCTYPE html>[\s\S]*?</html>").unwrap();
        let cleaned_spec = html_re.replace(&spec_content, "").into_owned();
        let spec_json = serde_json::to_string(&cleaned_spec).unwrap();

        let html = format!(
            r#"
<!doctype html>
<html>
  <head>
    <title>API Reference</title>
    <meta charset="utf-8" />
    <meta
      name="viewport"
      content="width=device-width, initial-scale=1" />
    <style>
      body {{
        margin: 0;
      }}
    </style>
  </head>
  <body>
    <script id="api-reference"></script>
    <script>
      var spec = {};
      var configuration = {{
        theme: 'default',
        spec: {{ content: spec }},
      }}
      document.getElementById('api-reference').dataset.configuration = JSON.stringify(configuration)
    </script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
  </body>
</html>
"#,
            spec_json
        );
        axum::response::Html(html).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
