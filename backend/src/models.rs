use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Server {
    pub id: i64,
    pub name: String,
    pub host: String,
    pub port: u16,
    #[serde(skip_serializing)]
    // Don't expose API key in list response by default? Or maybe user needs to see it? For security, skip.
    pub api_key: String,
    // created_at is strictly DB managed for now, or fetch if needed
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    // Add more fields as we identify them from the 1Panel response
    // For now, this is a best-guess structure
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ContainerOperationReq {
    pub names: Vec<String>,
    pub operation: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InspectReq {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Repository {
    pub id: i64,
    pub path: String,
    pub name: Option<String>,
    pub docker_image_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateDirectoryRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DirectoryResponse {
    pub path: String,
    pub docker_image_name: Option<String>,
    pub git_status: Option<crate::git::GitStatus>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateServerRequest {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ServerResponse {
    pub id: i64,
    pub name: String,
    pub host: String,
    pub port: u16,
}

// OS Status Response (from 1Panel API)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OsInfo {
    pub os: String,
    pub platform: String,
    #[serde(rename = "platformFamily")]
    pub platform_family: String,
    #[serde(rename = "kernelArch")]
    pub kernel_arch: String,
    #[serde(rename = "kernelVersion")]
    pub kernel_version: String,
    // diskSize might be large integer
    #[serde(rename = "diskSize")]
    pub disk_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<OsInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PushImageReq {
    #[serde(rename = "serverId")]
    pub server_id: i64,
    #[serde(rename = "imageTag")]
    pub image_tag: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateDockerConfigReq {
    pub path: String,
    pub docker_image_name: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct GetDockerConfigReq {
    pub path: String,
}
