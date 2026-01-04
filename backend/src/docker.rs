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

// Struct for build request
#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct DockerBuildRequest {
    pub path: String,
    pub image_name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct DockerImage {
    pub id: String,
    pub tags: Vec<String>,
    pub created: i64,
    pub size: i64,
}

pub async fn list_tags(image_name: &str) -> Result<Vec<DockerImage>, String> {
    let docker = Docker::connect_with_local_defaults().map_err(|e| e.to_string())?;

    // List images with filter by name
    use bollard::image::ListImagesOptions;
    use std::collections::HashMap;

    let mut filters: HashMap<String, Vec<String>> = HashMap::new();
    filters.insert("reference".to_string(), vec![image_name.to_string()]);

    let options = ListImagesOptions {
        filters,
        ..Default::default()
    };

    let images = docker
        .list_images(Some(options))
        .await
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for image in images {
        result.push(DockerImage {
            id: image.id.replace("sha256:", "").chars().take(12).collect(),
            tags: image.repo_tags,
            created: image.created,
            size: image.size,
        });
    }

    Ok(result)
}

// use futures_util::stream::StreamExt; // Removed

pub async fn build_image(req: DockerBuildRequest) -> Result<String, String> {
    // let docker = Docker::connect_with_local_defaults().map_err(|e| e.to_string())?; // Unused for Command based build

    // Prepare tarball
    // Bollard build_image requires a tarball of the context.
    // For simplicity, we might want to use std::process::Command for "docker build" if we want to avoid creating tarball manually in Rust.
    // Creating a tarball of a directory in Rust is doable but adds complexity.
    // Given "OnePanel-CI" likely runs on a machine with `docker` CLI available (since it connects to docker socket).
    // Using Command is often more robust for "build from folder" than streaming tarball if we don't want to implement tar logic.
    // However, if we want to stick to pure Rust/API...
    // Let's use Command for now as it's easier to implement "build from this directory".
    // Wait, `req.path` is local path.

    use std::process::Command;

    let full_tag = format!("{}:{}", req.image_name, req.version);
    let latest_tag = format!("{}:latest", req.image_name);

    let output = Command::new("docker")
        .arg("build")
        .arg("-t")
        .arg(&full_tag)
        .arg("-t")
        .arg(&latest_tag)
        .arg(".")
        .current_dir(&req.path)
        .output()
        .map_err(|e| format!("Failed to execute docker build: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
