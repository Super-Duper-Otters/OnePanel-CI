use async_trait::async_trait;
use rust_mcp_sdk::{
    macros,
    mcp_server::ServerHandler,
    schema::{
        CallToolError, CallToolRequestParams, CallToolResult, ListToolsResult,
        PaginatedRequestParams,
    },
    McpServer,
};

use std::sync::Arc;
use tracing::info;

use crate::docker;
use crate::models::Repository;

use crate::handlers::deploy;
use crate::onepanel::OnePanelClient;
use crate::state::AppState;
use regex::Regex;

// --- Tool Definitions ---

#[macros::mcp_tool(
    name = "list_projects",
    description = "List all configured CI repositories/projects"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, macros::JsonSchema)]
pub struct ListProjectsTool {}

#[macros::mcp_tool(
    name = "list_servers",
    description = "List all configured 1Panel servers."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, macros::JsonSchema)]
pub struct ListServersTool {}

#[macros::mcp_tool(
    name = "build_image",
    description = "Build a Docker image from a directory"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, macros::JsonSchema)]
pub struct BuildImageTool {
    pub path: String,
}

#[macros::mcp_tool(
    name = "build_and_deploy",
    description = "Build a Docker image from a project path and deploy it to the configured server."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, macros::JsonSchema)]
pub struct BuildAndDeployTool {
    pub path: String,
}

pub struct AppHandler {
    pub state: AppState,
}

impl AppHandler {
    pub async fn list_tools(&self) -> std::result::Result<ListToolsResult, String> {
        Ok(ListToolsResult {
            tools: vec![
                ListProjectsTool::tool(),
                ListServersTool::tool(),
                BuildImageTool::tool(),
                BuildAndDeployTool::tool(),
            ],
            meta: None,
            next_cursor: None,
        })
    }

    pub async fn call_tool(
        &self,
        name: String,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        info!("Calling tool: {}", name);

        match name.as_str() {
            "list_projects" => {
                let repos = crate::handlers::repository::list_repositories_inner(&self.state.db)
                    .await
                    .map_err(|e| CallToolError::from_message(e.to_string()))?;

                let text = serde_json::to_string_pretty(&repos).unwrap_or_default();
                Ok(CallToolResult::text_content(vec![text.into()]))
            }
            "list_servers" => {
                let servers = sqlx::query_as::<_, crate::models::Server>("SELECT * FROM servers")
                    .fetch_all(&*self.state.db)
                    .await
                    .map_err(|e| CallToolError::from_message(e.to_string()))?;

                let safe_servers: Vec<serde_json::Value> = servers
                    .into_iter()
                    .map(|s| {
                        serde_json::json!({
                            "id": s.id,
                            "name": s.name,
                            "host": s.host,
                            "port": s.port
                        })
                    })
                    .collect();

                let text = serde_json::to_string_pretty(&safe_servers).unwrap_or_default();
                Ok(CallToolResult::text_content(vec![text.into()]))
            }
            "build_image" => {
                info!("build_image: Parsing args for");
                let args_val = serde_json::Value::Object(arguments.unwrap_or_default());
                let args: BuildImageTool = serde_json::from_value(args_val).map_err(|e| {
                    CallToolError::from_message(format!("Invalid parameters: {}", e))
                })?;
                info!("build_image: Parsed args: {}", args.path);

                // Fire and forget
                let state_clone = self.state.clone();
                let args_clone = args;

                info!("build_image: Spawning background task");
                tokio::spawn(async move {
                    info!(
                        "task: Starting background build for path: {}",
                        args_clone.path
                    );

                    // 1. Verify path exists in DB
                    info!("task: Verifying path in DB");
                    let mut repo = sqlx::query_as::<_, Repository>(
                        "SELECT * FROM repositories WHERE path = ?",
                    )
                    .bind(&args_clone.path)
                    .fetch_optional(&*state_clone.db)
                    .await
                    .unwrap_or(None);

                    if repo.is_none() {
                        info!("task: Exact match not found, trying smart search");
                        // Fetch all and search manually
                        if let Ok(all_repos) =
                            sqlx::query_as::<_, Repository>("SELECT * FROM repositories")
                                .fetch_all(&*state_clone.db)
                                .await
                        {
                            let search_term = args_clone.path.to_lowercase().replace("\\", "/");
                            let search_term_trim = search_term.trim_end_matches('/');

                            for r in all_repos {
                                let r_path = r.path.to_lowercase().replace("\\", "/");
                                let r_path_trim = r_path.trim_end_matches('/');

                                // 1. Path match
                                if r_path_trim == search_term_trim {
                                    repo = Some(r);
                                    break;
                                }

                                // 2. Name match
                                if let Some(n) = &r.name {
                                    if n.to_lowercase() == search_term_trim {
                                        repo = Some(r);
                                        break;
                                    }
                                }

                                // 3. Folder name match (ends with /search_term)
                                if r_path_trim.ends_with(&format!("/{}", search_term_trim)) {
                                    repo = Some(r);
                                    break;
                                }
                            }
                        }
                    }

                    let repo = match repo {
                        Some(r) => {
                            info!("task: Found repo in DB: {} (id: {})", r.path, r.id);
                            r
                        }
                        None => {
                            info!("Build failed: Path not configured: {}", args_clone.path);
                            return;
                        }
                    };

                    // 2. Determine image name
                    let image_name = repo.docker_image_name.clone().unwrap_or_else(|| {
                        std::path::Path::new(&repo.path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    });

                    // 3. Determine version (auto-increment)
                    let tags = match docker::list_tags(&image_name).await {
                        Ok(t) => t,
                        Err(e) => {
                            info!("Build failed: List tags error: {}", e);
                            return;
                        }
                    };

                    let version_regex = Regex::new(r"^v?(\d+)(\.(\d+))?(\.(\d+))?$").unwrap();
                    let mut max_ver = (0, 0, 0); // major, minor, patch
                    let mut max_int_ver = 0;
                    let mut found_semver = false;

                    for img in tags {
                        for tag in img.tags {
                            if let Some(pos) = tag.rfind(':') {
                                let ver_str = &tag[pos + 1..];
                                if let Some(caps) = version_regex.captures(ver_str) {
                                    let major =
                                        caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
                                    let minor =
                                        caps.get(3).map_or(0, |m| m.as_str().parse().unwrap_or(0));
                                    let patch =
                                        caps.get(5).map_or(0, |m| m.as_str().parse().unwrap_or(0));

                                    // Poor man's version compare
                                    if major > max_ver.0
                                        || (major == max_ver.0 && minor > max_ver.1)
                                        || (major == max_ver.0
                                            && minor == max_ver.1
                                            && patch > max_ver.2)
                                    {
                                        max_ver = (major, minor, patch);
                                        found_semver = true;
                                    }
                                } else if let Ok(v) = ver_str.parse::<i32>() {
                                    if v > max_int_ver {
                                        max_int_ver = v;
                                    }
                                }
                            }
                        }
                    }

                    let new_version = if found_semver {
                        format!("v{}.{}.{}", max_ver.0, max_ver.1, max_ver.2 + 1)
                    } else if max_int_ver > 0 {
                        format!("{}", max_int_ver + 1)
                    } else {
                        "v1.0.0".to_string()
                    };

                    let req = docker::DockerBuildRequest {
                        path: repo.path.clone(),
                        image_name: image_name.clone(),
                        version: new_version.clone(),
                    };

                    match docker::build_image(req).await {
                        Ok(_) => info!(
                            "Background Build Successful for {}:{}",
                            image_name, new_version
                        ),
                        Err(e) => info!(
                            "Background Build Failed for {}:{}: {}",
                            image_name, new_version, e
                        ),
                    }
                    info!("task: Background build finished");
                });

                info!("build_image: Returning immediate success");
                Ok(CallToolResult::text_content(vec![
                    "Build started in background.".into(),
                ]))
            }
            "build_and_deploy" => {
                let args_val = serde_json::Value::Object(arguments.unwrap_or_default());
                let args: BuildAndDeployTool = serde_json::from_value(args_val).map_err(|e| {
                    CallToolError::from_message(format!("Invalid parameters: {}", e))
                })?;

                // Fire and forget
                let state_clone = self.state.clone();
                let args_clone = args;

                tokio::spawn(async move {
                    info!(
                        "Starting background build_and_deploy for path: {}",
                        args_clone.path
                    );

                    // 1. Verify path exists in DB
                    let mut repo = sqlx::query_as::<_, Repository>(
                        "SELECT * FROM repositories WHERE path = ?",
                    )
                    .bind(&args_clone.path)
                    .fetch_optional(&*state_clone.db)
                    .await
                    .unwrap_or(None);

                    if repo.is_none() {
                        info!("task: Exact match not found, trying smart search");
                        // Fetch all and search manually
                        if let Ok(all_repos) =
                            sqlx::query_as::<_, Repository>("SELECT * FROM repositories")
                                .fetch_all(&*state_clone.db)
                                .await
                        {
                            let search_term = args_clone.path.to_lowercase().replace("\\", "/");
                            let search_term_trim = search_term.trim_end_matches('/');

                            for r in all_repos {
                                let r_path = r.path.to_lowercase().replace("\\", "/");
                                let r_path_trim = r_path.trim_end_matches('/');

                                // 1. Path match
                                if r_path_trim == search_term_trim {
                                    repo = Some(r);
                                    break;
                                }

                                // 2. Name match
                                if let Some(n) = &r.name {
                                    if n.to_lowercase() == search_term_trim {
                                        repo = Some(r);
                                        break;
                                    }
                                }

                                // 3. Folder name match (ends with /search_term)
                                if r_path_trim.ends_with(&format!("/{}", search_term_trim)) {
                                    repo = Some(r);
                                    break;
                                }
                            }
                        }
                    }

                    let repo = match repo {
                        Some(r) => {
                            info!("task: Found repo in DB: {} (id: {})", r.path, r.id);
                            r
                        }
                        None => {
                            info!("Deploy failed: Path not configured: {}", args_clone.path);
                            return;
                        }
                    };

                    let server_id = match repo.default_server_id {
                        Some(id) => id,
                        None => {
                            info!("Deploy failed: Default server not configured");
                            return;
                        }
                    };

                    let compose_path = match repo.default_compose_path {
                        Some(p) => p,
                        None => {
                            info!("Deploy failed: Default compose path not configured");
                            return;
                        }
                    };

                    // 2. SemVer Logic
                    let image_name = repo.docker_image_name.clone().unwrap_or_else(|| {
                        std::path::Path::new(&repo.path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    });

                    let tags = match docker::list_tags(&image_name).await {
                        Ok(t) => t,
                        Err(e) => {
                            info!("Deploy failed: List tags error: {}", e);
                            return;
                        }
                    };

                    let version_regex = Regex::new(r"^v?(\d+)(\.(\d+))?(\.(\d+))?$").unwrap();
                    let mut max_ver = (0, 0, 0); // major, minor, patch
                    let mut max_int_ver = 0;
                    let mut found_semver = false;

                    for img in tags {
                        for tag in img.tags {
                            if let Some(pos) = tag.rfind(':') {
                                let ver_str = &tag[pos + 1..];
                                if let Some(caps) = version_regex.captures(ver_str) {
                                    let major =
                                        caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
                                    let minor =
                                        caps.get(3).map_or(0, |m| m.as_str().parse().unwrap_or(0));
                                    let patch =
                                        caps.get(5).map_or(0, |m| m.as_str().parse().unwrap_or(0));

                                    // Poor man's version compare
                                    if major > max_ver.0
                                        || (major == max_ver.0 && minor > max_ver.1)
                                        || (major == max_ver.0
                                            && minor == max_ver.1
                                            && patch > max_ver.2)
                                    {
                                        max_ver = (major, minor, patch);
                                        found_semver = true;
                                    }
                                } else if let Ok(v) = ver_str.parse::<i32>() {
                                    if v > max_int_ver {
                                        max_int_ver = v;
                                    }
                                }
                            }
                        }
                    }

                    let new_version = if found_semver {
                        format!("v{}.{}.{}", max_ver.0, max_ver.1, max_ver.2 + 1)
                    } else if max_int_ver > 0 {
                        format!("{}", max_int_ver + 1)
                    } else {
                        "v1.0.0".to_string()
                    };

                    // 3. Build
                    let req = docker::DockerBuildRequest {
                        path: repo.path.clone(),
                        image_name: image_name.clone(),
                        version: new_version.clone(),
                    };
                    if let Err(e) = docker::build_image(req).await {
                        info!("Deploy failed: Build error: {}", e);
                        return;
                    }

                    // 4. Push
                    let full_tag = format!("{}:{}", image_name, new_version);
                    if let Err(e) =
                        deploy::push_image_to_server_inner(&state_clone.db, server_id, &full_tag)
                            .await
                    {
                        info!("Deploy failed: Push error: {}", e);
                        return;
                    }

                    // 5. Deploy (Update Compose)
                    let server = match sqlx::query_as::<_, crate::models::Server>(
                        "SELECT * FROM servers WHERE id = ?",
                    )
                    .bind(server_id)
                    .fetch_one(&*state_clone.db)
                    .await
                    {
                        Ok(s) => s,
                        Err(e) => {
                            info!("Deploy failed: Fetch server error: {}", e);
                            return;
                        }
                    };

                    // Find stack name by matching compose path
                    let composes = match OnePanelClient::list_composes(
                        &server.host,
                        server.port,
                        &server.api_key,
                    )
                    .await
                    {
                        Ok(c) => c,
                        Err(e) => {
                            info!("Deploy failed: List composes error: {}", e);
                            return;
                        }
                    };

                    let mut stack_name = None;
                    for item in composes {
                        if let Some(path) = item.get("path").and_then(|s| s.as_str()) {
                            if path == compose_path {
                                if let Some(name) = item.get("name").and_then(|s| s.as_str()) {
                                    stack_name = Some(name.to_string());
                                    break;
                                }
                            }
                        }
                    }

                    let stack_name = match stack_name {
                        Some(n) => n,
                        None => {
                            info!(
                                "Deploy failed: Could not find 1Panel stack for path: {}",
                                compose_path
                            );
                            return;
                        }
                    };

                    // Read File
                    let content = match OnePanelClient::read_file(
                        &server.host,
                        server.port,
                        &server.api_key,
                        &compose_path,
                    )
                    .await
                    {
                        Ok(c) => c,
                        Err(e) => {
                            info!("Deploy failed: Read file error: {}", e);
                            return;
                        }
                    };

                    // Replace Image Tag
                    let pattern = format!(r"(image:\s*{}:)[\w.-]+", regex::escape(&image_name));
                    let re = match Regex::new(&pattern) {
                        Ok(r) => r,
                        Err(e) => {
                            info!("Deploy failed: Regex error: {}", e);
                            return;
                        }
                    };
                    let new_line = format!("${{1}}{}", new_version);
                    let new_content = re.replace_all(&content, new_line.as_str());

                    // Update Compose
                    if let Err(e) = OnePanelClient::update_compose(
                        &server.host,
                        server.port,
                        &server.api_key,
                        &stack_name,
                        &compose_path,
                        &new_content,
                    )
                    .await
                    {
                        info!("Deploy failed: Update compose error: {}", e);
                        return;
                    }

                    // Restart
                    if let Err(e) = OnePanelClient::operate_compose(
                        &server.host,
                        server.port,
                        &server.api_key,
                        &stack_name,
                        &compose_path,
                        "up",
                    )
                    .await
                    {
                        info!("Deploy failed: Operate compose error: {}", e);
                        return;
                    }

                    info!(
                        "Background Deploy Successful for {}:version {}",
                        server.name, new_version
                    );
                });

                Ok(CallToolResult::text_content(vec![
                    "Build and Deploy started in background.".into(),
                ]))
            }

            _ => Err(CallToolError::unknown_tool(name)),
        }
    }
}

#[async_trait]
impl ServerHandler for AppHandler {
    async fn handle_list_tools_request(
        &self,
        _request: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, rust_mcp_sdk::schema::RpcError> {
        self.list_tools()
            .await
            .map_err(|e| rust_mcp_sdk::schema::RpcError {
                code: 0,
                message: e,
                data: None,
            })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        self.call_tool(params.name, params.arguments).await
    }
}
