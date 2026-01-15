use crate::mcp_server::AppHandler;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    Json,
};
use futures_util::StreamExt;
use rust_mcp_sdk::schema::{
    Implementation, InitializeResult, ProtocolVersion, ServerCapabilities, ServerCapabilitiesTools,
};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

// Custom JSON-RPC Request to avoid SDK dependency
#[derive(serde::Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

// Custom JSON-RPC Response
#[derive(serde::Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(serde::Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

pub async fn sse_handler(State(state): State<AppState>) -> impl IntoResponse {
    let (tx, rx) = mpsc::unbounded_channel::<String>();
    let session_id = Uuid::new_v4().to_string();

    state
        .mcp_sessions
        .write()
        .await
        .insert(session_id.clone(), tx.clone());

    let stream = UnboundedReceiverStream::new(rx)
        .map(|msg| Ok::<Event, axum::Error>(Event::default().data(msg)));

    // Use absolute URL to avoid client-side resolution issues
    let post_url = format!("http://127.0.0.1:3000/mcp?sessionId={}", session_id);
    let initial_event = Ok::<Event, axum::Error>(Event::default().event("endpoint").data(post_url));

    Sse::new(tokio_stream::iter(vec![initial_event]).chain(stream))
        .keep_alive(axum::response::sse::KeepAlive::default())
}

#[derive(serde::Deserialize)]
pub struct McpQueryParams {
    #[serde(default, rename = "sessionId")]
    session_id: Option<String>,
}

pub async fn post_handler(
    State(state): State<AppState>,
    Query(_params): Query<McpQueryParams>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let handler = AppHandler {
        state: state.clone(),
    };

    match request.method.as_str() {
        "initialize" => {
            let result = InitializeResult {
                server_info: Implementation {
                    name: "OnePanel-CI-MCP-Native".into(),
                    version: env!("APP_VERSION").into(),
                    title: Some("OnePanel CI MCP Native".into()),
                    description: Some("Control OnePanel CI via MCP (Native)".into()),
                    icons: vec![],
                    website_url: None,
                },
                capabilities: ServerCapabilities {
                    tools: Some(ServerCapabilitiesTools {
                        list_changed: Some(false),
                    }),
                    ..Default::default()
                },
                protocol_version: ProtocolVersion::V2024_11_05.into(),
                instructions: None,
                meta: None,
            };

            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::to_value(result).unwrap()),
                error: None,
            };

            Json(response)
        }
        "notifications/initialized" => Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({})),
            error: None,
        }),
        "ping" => {
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({})),
                error: None,
            };
            Json(response)
        }
        "tools/list" => {
            // DIRECT CALL to list_tools
            let res = handler.list_tools().await;
            match res {
                Ok(tools) => {
                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(serde_json::to_value(tools).unwrap()),
                        error: None,
                    };
                    Json(response)
                }
                Err(e) => Json(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: e,
                        data: None,
                    }),
                }),
            }
        }
        "tools/call" => {
            let params_val = request.params.unwrap_or(json!({}));
            // Parse manually or use serde if wrapper struct is available.
            // CallToolRequestParams has `name` and `arguments`.
            // We can use serde_json to deserialize ONLY if CallToolRequestParams is clean.
            // Or map Manually.

            let tool_name = params_val
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let tool_args = params_val
                .get("arguments")
                .and_then(|v| v.as_object())
                .cloned();

            if let Some(name) = tool_name {
                // DIRECT CALL to call_tool
                let res = handler.call_tool(name, tool_args).await;
                match res {
                    Ok(result) => {
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: Some(serde_json::to_value(result).unwrap()),
                            error: None,
                        };
                        Json(response)
                    }
                    Err(e) => Json(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32603,
                            message: format!("{}", e),
                            data: None,
                        }),
                    }),
                }
            } else {
                Json(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing 'name' parameter".to_string(),
                        data: None,
                    }),
                })
            }
        }
        _ => Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        }),
    }
}
