use async_trait::async_trait;
use rust_mcp_sdk::{
    error::SdkResult,
    macros,
    mcp_server::{hyper_server, HyperServerOptions, ServerHandler},
    schema::{
        CallToolError, CallToolRequestParams, CallToolResult, Implementation, InitializeResult,
        ListToolsResult, PaginatedRequestParams, ProtocolVersion, ServerCapabilities,
        ServerCapabilitiesTools,
    },
    McpServer, ToMcpServerHandler,
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

// --- Handler ---

struct AppHandler {
    state: AppState,
}

#[async_trait]
impl ServerHandler for AppHandler {
    async fn handle_list_tools_request(
        &self,
        _request: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, rust_mcp_sdk::schema::RpcError> {
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

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        info!("Calling tool: {}", params.name);

        match params.name.as_str() {
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
                let args_val = serde_json::Value::Object(params.arguments.unwrap_or_default());
                let args: BuildImageTool = serde_json::from_value(args_val).map_err(|e| {
                    CallToolError::from_message(format!("Invalid parameters: {}", e))
                })?;

                // 1. Verify path exists in DB
                let repo =
                    sqlx::query_as::<_, Repository>("SELECT * FROM repositories WHERE path = ?")
                        .bind(&args.path)
                        .fetch_optional(&*self.state.db)
                        .await
                        .map_err(|e| CallToolError::from_message(e.to_string()))?
                        .ok_or_else(|| {
                            CallToolError::from_message(
                                "Path not configured in OnePanel-CI. Please add it first."
                                    .to_string(),
                            )
                        })?;

                // 2. Determine image name
                let image_name = repo.docker_image_name.unwrap_or_else(|| {
                    std::path::Path::new(&args.path)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                });

                // 3. Determine version (auto-increment)
                let tags = docker::list_tags(&image_name)
                    .await
                    .map_err(|e| CallToolError::from_message(e))?;

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
                    path: args.path,
                    image_name: image_name.clone(),
                    version: new_version.clone(),
                };

                let _ = docker::build_image(req)
                    .await
                    .map_err(|e| CallToolError::from_message(e))?;

                Ok(CallToolResult::text_content(vec![format!(
                    "Build Successful for {}:{}",
                    image_name, new_version
                )
                .into()]))
            }
            "build_and_deploy" => {
                let args_val = serde_json::Value::Object(params.arguments.unwrap_or_default());
                let args: BuildAndDeployTool = serde_json::from_value(args_val).map_err(|e| {
                    CallToolError::from_message(format!("Invalid parameters: {}", e))
                })?;

                // 1. Validate & Config
                let repo =
                    sqlx::query_as::<_, Repository>("SELECT * FROM repositories WHERE path = ?")
                        .bind(&args.path)
                        .fetch_optional(&*self.state.db)
                        .await
                        .map_err(|e| CallToolError::from_message(e.to_string()))?
                        .ok_or_else(|| {
                            CallToolError::from_message(
                                "Path not configured in OnePanel-CI.".to_string(),
                            )
                        })?;

                let server_id = repo.default_server_id.ok_or_else(|| {
                    CallToolError::from_message(
                        "Default server not configured for this project.".to_string(),
                    )
                })?;

                let compose_path = repo.default_compose_path.ok_or_else(|| {
                    CallToolError::from_message(
                        "Default compose path not configured for this project.".to_string(),
                    )
                })?;

                // 2. SemVer Logic
                let image_name = repo.docker_image_name.unwrap_or_else(|| {
                    std::path::Path::new(&args.path)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                });

                let tags = docker::list_tags(&image_name)
                    .await
                    .map_err(|e| CallToolError::from_message(e))?;

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
                    path: args.path.clone(),
                    image_name: image_name.clone(),
                    version: new_version.clone(),
                };
                let _ = docker::build_image(req)
                    .await
                    .map_err(|e| CallToolError::from_message(format!("Build failed: {}", e)))?;

                // 4. Push
                let full_tag = format!("{}:{}", image_name, new_version);
                deploy::push_image_to_server_inner(&self.state.db, server_id, &full_tag)
                    .await
                    .map_err(|e| CallToolError::from_message(format!("Push failed: {}", e)))?;

                // 5. Deploy (Update Compose)
                let server = sqlx::query_as::<_, crate::models::Server>(
                    "SELECT * FROM servers WHERE id = ?",
                )
                .bind(server_id)
                .fetch_one(&*self.state.db)
                .await
                .map_err(|e| CallToolError::from_message(format!("Failed to get server: {}", e)))?;

                // Find stack name by matching compose path
                let composes =
                    OnePanelClient::list_composes(&server.host, server.port, &server.api_key)
                        .await
                        .map_err(|e| {
                            CallToolError::from_message(format!("List composes failed: {}", e))
                        })?;

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

                let stack_name = stack_name.ok_or_else(|| {
                    CallToolError::from_message(format!(
                        "Could not find 1Panel stack for path: {}",
                        compose_path
                    ))
                })?;

                // Read File
                let content = OnePanelClient::read_file(
                    &server.host,
                    server.port,
                    &server.api_key,
                    &compose_path,
                )
                .await
                .map_err(|e| CallToolError::from_message(format!("Read compose failed: {}", e)))?;

                // Replace Image Tag
                // Regex to find image: image_name:old_tag
                // We use image_name (e.g. "my-repo")
                // Pattern: `image:\s*my-repo:[\w.-]+`
                // But image_name might contain slashes. needed escaping?
                let pattern = format!(r"(image:\s*{}:)[\w.-]+", regex::escape(&image_name));
                let re = Regex::new(&pattern)
                    .map_err(|e| CallToolError::from_message(format!("Regex error: {}", e)))?;
                let new_line = format!("${{1}}{}", new_version);
                let new_content = re.replace_all(&content, new_line.as_str());

                // Update Compose
                OnePanelClient::update_compose(
                    &server.host,
                    server.port,
                    &server.api_key,
                    &stack_name,
                    &compose_path,
                    &new_content,
                )
                .await
                .map_err(|e| {
                    CallToolError::from_message(format!("Update compose failed: {}", e))
                })?;

                // Restart
                OnePanelClient::operate_compose(
                    &server.host,
                    server.port,
                    &server.api_key,
                    &stack_name,
                    &compose_path,
                    "up",
                )
                .await
                .map_err(|e| {
                    CallToolError::from_message(format!("Restart compose failed: {}", e))
                })?;

                Ok(CallToolResult::text_content(vec![format!(
                    "Successfully built, pushed to {}, and deployed version {}",
                    server.name, new_version
                )
                .into()]))
            }

            _ => Err(CallToolError::unknown_tool(params.name)),
        }
    }
}

pub async fn start_mcp_server(state: AppState) -> SdkResult<()> {
    let server_info = InitializeResult {
        server_info: Implementation {
            name: "OnePanel-CI-MCP".into(),
            version: env!("APP_VERSION").into(),
            title: Some("OnePanel CI MCP Server".into()),
            description: Some("Control OnePanel CI via MCP".into()),
            icons: vec![], // Add icon if needed
            website_url: None,
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools {
                list_changed: Some(false),
            }),
            ..Default::default()
        },
        protocol_version: ProtocolVersion::V2025_11_25.into(),
        instructions: None,
        meta: None,
    };

    // Explicitly import the trait and use it
    let handler = AppHandler { state }.to_mcp_server_handler();

    let server = hyper_server::create_server(
        server_info,
        handler,
        HyperServerOptions {
            host: "127.0.0.1".to_string(), // Listen on all interfaces if needed, or localhost
            port: 3001,
            sse_support: true,
            ..Default::default()
        },
    );

    println!("Starting MCP Server on http://127.0.0.1:3001/sse");
    server.start().await?;

    Ok(())
}
