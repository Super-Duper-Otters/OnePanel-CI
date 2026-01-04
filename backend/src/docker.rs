use bollard::Docker;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct DockerInfo {
    pub version: String,
    pub containers: usize,
    pub images: usize,
    pub status: String,
}

pub async fn get_docker_info() -> Result<DockerInfo, String> {
    // Attempt to connect to local docker
    let docker = Docker::connect_with_local_defaults().map_err(|e| e.to_string())?;

    let version = docker.version().await.map_err(|e| e.to_string())?;
    let info = docker.info().await.map_err(|e| e.to_string())?;

    // Using list_images and list_containers for counts if info doesn't provide them directly cleanly
    // or just rely on Info struct from bollard which has Containers, Images.
    // Use info fields directly.
    let containers = info.containers.unwrap_or(0) as usize;
    let images = info.images.unwrap_or(0) as usize;

    let ver_str = version.version.unwrap_or_else(|| "Unknown".to_string());

    Ok(DockerInfo {
        version: ver_str,
        containers,
        images,
        status: "Connected".to_string(),
    })
}
