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

use crate::state::AppState;

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
            tools: vec![ListProjectsTool::tool(), ListServersTool::tool()],
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
