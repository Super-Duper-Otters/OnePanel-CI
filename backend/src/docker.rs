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
    pub is_used: bool,
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
            is_used: false, // Default to false for tags list, or we could check usage here too but less critical
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

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ContainerSummary {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub state: String,
    pub status: String,
}

pub async fn list_containers() -> Result<Vec<ContainerSummary>, String> {
    let docker = Docker::connect_with_local_defaults().map_err(|e| e.to_string())?;
    use bollard::container::ListContainersOptions;
    let options = ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    };

    let containers = docker
        .list_containers(Some(options))
        .await
        .map_err(|e| e.to_string())?;

    let result = containers
        .into_iter()
        .map(|c| ContainerSummary {
            id: c.id.unwrap_or_default().chars().take(12).collect(),
            names: c.names.unwrap_or_default(),
            image: c.image.unwrap_or_default(),
            state: c.state.unwrap_or_default(),
            status: c.status.unwrap_or_default(),
        })
        .collect();

    Ok(result)
}

pub async fn start_container(id: &str) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults().map_err(|e| e.to_string())?;
    docker
        .start_container::<String>(id, None)
        .await
        .map_err(|e| e.to_string())
}

pub async fn stop_container(id: &str) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults().map_err(|e| e.to_string())?;
    docker
        .stop_container(id, None)
        .await
        .map_err(|e| e.to_string())
}

pub async fn remove_container(id: &str) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults().map_err(|e| e.to_string())?;
    docker
        .remove_container(
            id,
            Some(bollard::container::RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_container_logs(id: &str) -> Result<String, String> {
    let docker = Docker::connect_with_local_defaults().map_err(|e| e.to_string())?;
    use bollard::container::LogsOptions;
    use futures_util::TryStreamExt;

    let options = Some(LogsOptions::<String> {
        stdout: true,
        stderr: true,
        tail: "100".to_string(), // Get last 100 lines
        ..Default::default()
    });

    let streams = docker.logs(id, options);
    // Accumulate the logs
    let mut logs = String::new();
    let mut stream = streams;

    while let Ok(Some(output)) = stream.try_next().await {
        logs.push_str(&output.to_string());
    }

    Ok(logs)
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct PullImageRequest {
    pub image: String,
}

pub async fn pull_image(image_name: &str) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults().map_err(|e| e.to_string())?;
    use bollard::image::CreateImageOptions;
    use futures_util::TryStreamExt;

    let options = Some(CreateImageOptions {
        from_image: image_name,
        ..Default::default()
    });

    // Create the image (pull)
    let mut stream = docker.create_image(options, None, None);

    // Drive the stream to completion
    while let Ok(Some(_)) = stream.try_next().await {
        // We could forward progress here if we had a websocket
    }

    Ok(())
}

pub async fn remove_image(id: &str) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults().map_err(|e| e.to_string())?;
    use bollard::image::RemoveImageOptions;

    let options = Some(RemoveImageOptions {
        force: true,
        ..Default::default()
    });

    docker
        .remove_image(id, options, None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn list_images() -> Result<Vec<DockerImage>, String> {
    let docker = Docker::connect_with_local_defaults().map_err(|e| e.to_string())?;
    use bollard::container::ListContainersOptions;
    use bollard::image::ListImagesOptions;
    use std::collections::HashSet;

    let options = ListImagesOptions::<String> {
        ..Default::default()
    };

    let images = docker
        .list_images(Some(options))
        .await
        .map_err(|e| e.to_string())?;

    // Get all containers to check usage
    let container_options = ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    };
    let containers = docker
        .list_containers(Some(container_options))
        .await
        .map_err(|e| e.to_string())?;

    let mut used_images = HashSet::new();
    for container in containers {
        if let Some(image_id) = container.image_id {
            used_images.insert(
                image_id
                    .replace("sha256:", "")
                    .chars()
                    .take(12)
                    .collect::<String>(),
            );
            // Also add full id without prefix
            used_images.insert(container.image.unwrap_or_default());
        }
    }

    let mut result = Vec::new();
    for image in images {
        let id_short: String = image.id.replace("sha256:", "").chars().take(12).collect();
        // Check if any tag matches used images or ID matches
        let is_used = used_images.contains(&id_short)
            || image.repo_tags.iter().any(|tag| used_images.contains(tag));

        result.push(DockerImage {
            id: id_short,
            tags: image.repo_tags,
            created: image.created,
            size: image.size,
            is_used,
        });
    }

    Ok(result)
}

pub async fn prune_images() -> Result<String, String> {
    use std::process::Command;

    let output = Command::new("docker")
        .arg("image")
        .arg("prune")
        .arg("-f")
        .output()
        .map_err(|e| format!("Failed to execute docker prune: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
