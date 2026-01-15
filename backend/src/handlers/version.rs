use axum::{response::IntoResponse, Json};
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
}

#[derive(Debug, Serialize)]
struct VersionResponse {
    version: String,
    latest_version: Option<String>,
    update_available: bool,
}

pub async fn get_version() -> impl IntoResponse {
    let current_version = env!("APP_VERSION").to_string();
    let client = reqwest::Client::new();

    // Fetch latest release from GitHub
    let latest_release = client
        .get("https://api.github.com/repos/Super-Duper-Otters/OnePanel-CI/releases/latest")
        .header(USER_AGENT, "OnePanel-CI")
        .send()
        .await;

    let mut latest_version = None;
    let mut update_available = false;

    if let Ok(resp) = latest_release {
        if let Ok(release) = resp.json::<GithubRelease>().await {
            let tag = release.tag_name;
            latest_version = Some(tag.clone());

            // Simple version comparison
            let current_clean = current_version.trim_start_matches('v');
            let latest_clean = tag.trim_start_matches('v');

            if latest_clean != current_clean {
                // You might want a better semver comparison here ideally,
                // but string inequality + assumption that latest is usually newer works for simple cases
                // or if you want to be strict, only set update_available if latest > current.
                update_available = true;
            }
        }
    }

    Json(VersionResponse {
        version: current_version,
        latest_version,
        update_available,
    })
}
