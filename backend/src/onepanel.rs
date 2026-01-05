use crate::models::DashboardResponse;
use anyhow::{anyhow, Result};
use chrono::Utc;
#[allow(unused_imports)]
use reqwest::multipart;
use reqwest::Client;
use std::path::Path;

pub struct OnePanelClient;

impl OnePanelClient {
    #[allow(dead_code)]
    pub async fn test_connection(host: &str, port: u16, api_key: &str) -> Result<String> {
        let client = Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_else(|_| Client::new());

        let timestamp = Utc::now().timestamp();

        let token_raw = format!("1panel{}{}", api_key, timestamp);
        let token_digest = md5::compute(token_raw.as_bytes());
        let token_str = format!("{:x}", token_digest);

        let url = format!("http://{}:{}/api/v1/system/info", host, port);

        let res = client
            .get(&url)
            .header("1Panel-Token", token_str)
            .header("1Panel-Timestamp", timestamp.to_string())
            .send()
            .await?;

        if res.status().is_success() {
            Ok("Connection successful".to_string())
        } else if res.status() == reqwest::StatusCode::UNAUTHORIZED {
            Err(anyhow!("Authentication failed"))
        } else {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            Err(anyhow!("Request failed: {} - {}", status, body))
        }
    }

    pub async fn get_os_info(host: &str, port: u16, api_key: &str) -> Result<DashboardResponse> {
        let client = Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_else(|_| Client::new());

        let timestamp = Utc::now().timestamp();

        let api_key = api_key.trim();
        let host_clean = host
            .trim()
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .trim_end_matches('/');

        let token_raw = format!("1panel{}{}", api_key, timestamp);
        let token_digest = md5::compute(token_raw.as_bytes());
        let token_str = format!("{:x}", token_digest);

        // Assume HTTP for now as user example was HTTP. TODO: Support HTTPS toggle.
        let url = format!("http://{}:{}/api/v1/dashboard/base/os", host_clean, port);

        println!("Requesting URL: {}", url);

        let res = client
            .get(&url)
            .header("1Panel-Token", token_str)
            .header("1Panel-Timestamp", timestamp.to_string())
            .send()
            .await?;

        if res.status().is_success() {
            let body_text = res.text().await?;
            println!("Response Body: {}", body_text);

            let dashboard_res: Result<DashboardResponse, _> = serde_json::from_str(&body_text);
            match dashboard_res {
                Ok(data) => Ok(data),
                Err(e) => {
                    println!("Deserialization Error: {}", e);
                    Err(anyhow!("Deserialization failed: {}", e))
                }
            }
        } else {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            println!("Request failed. Status: {}, Body: {}", status, body);
            Err(anyhow!("Request failed: {} - {}", status, body))
        }
    }

    pub async fn list_containers(
        host: &str,
        port: u16,
        api_key: &str,
    ) -> Result<Vec<serde_json::Value>> {
        let client = Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_else(|_| Client::new());

        let timestamp = Utc::now().timestamp();
        let api_key = api_key.trim();
        let host_clean = host
            .trim()
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .trim_end_matches('/');

        let token_raw = format!("1panel{}{}", api_key, timestamp);
        let token_digest = md5::compute(token_raw.as_bytes());
        let token_str = format!("{:x}", token_digest);

        let url = format!("http://{}:{}/api/v1/containers/search", host_clean, port);

        let payload = serde_json::json!({
            "page": 1,
            "pageSize": 100,
            "name": "",
            "state": "all",
            "orderBy": "created_at",
            "order": "descending"
        });

        let res = client
            .post(&url)
            .header("1Panel-Token", token_str)
            .header("1Panel-Timestamp", timestamp.to_string())
            .json(&payload)
            .send()
            .await?;

        if res.status().is_success() {
            let text = res.text().await?;
            println!("List Containers Raw Response: {}", text);
            // Parse as Value to handle wrapper
            let json: serde_json::Value = serde_json::from_str(&text)
                .map_err(|e| anyhow!("Failed to parse response JSON: {} | Body: {}", e, text))?;

            // Check code
            if let Some(code) = json.get("code").and_then(|c| c.as_i64()) {
                if code != 200 {
                    let msg = json
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");
                    return Err(anyhow!("API returned error code {}: {}", code, msg));
                }
            }

            // Extract data
            let data = json
                .get("data")
                .ok_or_else(|| anyhow!("Missing data field in response"))?;

            println!("Data extracted. Is array? {}", data.is_array());

            // Extract items logic: search endpoint return structure might differ slightly or be standard page wrapper
            let items = if let Some(items) = data.get("items").and_then(|i| i.as_array()) {
                println!("Found items in pagination");
                items.clone()
            } else if let Some(arr) = data.as_array() {
                println!("Found direct array");
                arr.clone()
            } else {
                return Err(anyhow!("Data field is not an array or page result"));
            };

            println!("Items count: {}", items.len());

            // Return items directly as values
            Ok(items)
        } else {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            Err(anyhow!("Search containers failed: {} - {}", status, body))
        }
    }

    pub async fn operate_container(
        host: &str,
        port: u16,
        api_key: &str,
        names: Vec<String>,
        operation: String,
    ) -> Result<()> {
        let client = Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_else(|_| Client::new());

        let timestamp = Utc::now().timestamp();
        let api_key = api_key.trim();
        let host_clean = host
            .trim()
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .trim_end_matches('/');

        let token_raw = format!("1panel{}{}", api_key, timestamp);
        let token_digest = md5::compute(token_raw.as_bytes());
        let token_str = format!("{:x}", token_digest);

        let url = format!("http://{}:{}/api/v1/containers/operate", host_clean, port);

        let payload = serde_json::json!({
            "names": names,
            "operation": operation
        });

        let res = client
            .post(&url)
            .header("1Panel-Token", token_str)
            .header("1Panel-Timestamp", timestamp.to_string())
            .json(&payload)
            .send()
            .await?;

        if res.status().is_success() {
            let text = res.text().await?;
            // Parse wrapper. Even for operate, we should check code.
            let json: serde_json::Value =
                serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
            if let Some(code) = json.get("code").and_then(|c| c.as_i64()) {
                if code != 200 {
                    let msg = json
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");
                    return Err(anyhow!("API returned error code {}: {}", code, msg));
                }
            }
            Ok(())
        } else {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            Err(anyhow!("Operate container failed: {} - {}", status, body))
        }
    }

    pub async fn get_container_logs(
        host: &str,
        port: u16,
        api_key: &str,
        container: String,
    ) -> Result<String> {
        let client = Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_else(|_| Client::new());

        let timestamp = Utc::now().timestamp();
        let api_key = api_key.trim();
        let host_clean = host
            .trim()
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .trim_end_matches('/');

        let token_raw = format!("1panel{}{}", api_key, timestamp);
        let token_digest = md5::compute(token_raw.as_bytes());
        let token_str = format!("{:x}", token_digest);

        // search/log is POST
        let url = format!(
            "http://{}:{}/api/v1/containers/search/log?container={}&tail=100",
            host_clean, port, container
        );

        let res = client
            .post(&url)
            .header("1Panel-Token", token_str)
            .header("1Panel-Timestamp", timestamp.to_string())
            .send()
            .await?;

        if res.status().is_success() {
            let text = res.text().await?;
            let json: serde_json::Value =
                serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);

            // If it's pure text logs it won't be valid JSON (or it will be just a string).
            // But if it's wrapped JSON...
            // Let's check if it has "code" and "data".
            if let Some(code) = json.get("code").and_then(|c| c.as_i64()) {
                if code == 200 {
                    // return data (string)
                    if let Some(data) = json.get("data").and_then(|d| d.as_str()) {
                        return Ok(data.to_string());
                    }
                    // If data is not string, maybe it's not wrapped or logic differs?
                    // Falling back to returning whole text or debug.
                } else {
                    let msg = json
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");
                    return Err(anyhow!("API returned error code {}: {}", code, msg));
                }
            }

            // If fallback (not JSON or no code), usually implies raw logs or we failed to parse JSON
            // But if we failed to parse JSON (unwrap_or Null), we return text.
            if json.is_null() {
                Ok(text)
            } else {
                // It was valid JSON but didn't have code/data? Treat as logs?
                // Or maybe data was just missing.
                Ok(text)
            }
        } else {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            Err(anyhow!("Get logs failed: {} - {}", status, body))
        }
    }

    pub async fn upload_file(
        host: &str,
        port: u16,
        api_key: &str,
        file_path: &Path,
        remote_dir: &str,
    ) -> Result<String> {
        let client = Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_else(|_| Client::new());

        let timestamp = Utc::now().timestamp();
        let api_key = api_key.trim();
        let host_clean = host
            .trim()
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .trim_end_matches('/');

        let token_raw = format!("1panel{}{}", api_key, timestamp);
        let token_digest = md5::compute(token_raw.as_bytes());
        let token_str = format!("{:x}", token_digest);

        let url = format!("http://{}:{}/api/v1/files/upload", host_clean, port);

        // Prepare multipart
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid filename"))?
            .to_string();

        let file_content = tokio::fs::read(file_path).await?;
        let part_file = multipart::Part::bytes(file_content).file_name(file_name.clone());

        let form = multipart::Form::new()
            .part("file", part_file)
            .part("path", multipart::Part::text(remote_dir.to_string()))
            .part("overwrite", multipart::Part::text("true"));

        let res = client
            .post(&url)
            .header("1Panel-Token", token_str)
            .header("1Panel-Timestamp", timestamp.to_string())
            .multipart(form)
            .send()
            .await?;

        if res.status().is_success() {
            let text = res.text().await?;
            let json: serde_json::Value =
                serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);

            // Check for error code in successful http response
            if let Some(code) = json.get("code").and_then(|c| c.as_i64()) {
                if code != 200 {
                    let msg = json
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");
                    return Err(anyhow!("Upload failed (API {}): {}", code, msg));
                }
            }

            if let Some(data) = json.get("data").and_then(|d| d.as_str()) {
                Ok(data.to_string())
            } else {
                // If data is null, assume successes and construct path
                // But usually 1Panel might return something?
                // If null, we construct it: remote_dir + / + filename
                let full_path = format!("{}/{}", Self::remote_path_clean(remote_dir), file_name);
                Ok(full_path)
            }
        } else {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            Err(anyhow!("Upload file failed: {} - {}", status, body))
        }
    }

    // Helper helper
    fn remote_path_clean(p: &str) -> String {
        if p.ends_with('/') {
            p[..p.len() - 1].to_string()
        } else {
            p.to_string()
        }
    }

    pub async fn load_image(host: &str, port: u16, api_key: &str, remote_path: &str) -> Result<()> {
        let client = Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_else(|_| Client::new());

        let timestamp = Utc::now().timestamp();
        let api_key = api_key.trim();
        let host_clean = host
            .trim()
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .trim_end_matches('/');

        let token_raw = format!("1panel{}{}", api_key, timestamp);
        let token_digest = md5::compute(token_raw.as_bytes());
        let token_str = format!("{:x}", token_digest);

        let url = format!(
            "http://{}:{}/api/v1/containers/image/load",
            host_clean, port
        );

        let payload = serde_json::json!({
            "path": remote_path
        });

        let res = client
            .post(&url)
            .header("1Panel-Token", token_str)
            .header("1Panel-Timestamp", timestamp.to_string())
            .json(&payload)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            Err(anyhow!("Load image failed: {} - {}", status, body))
        }
    }
}
