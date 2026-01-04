use crate::models::DashboardResponse;
use anyhow::{anyhow, Result};
use chrono::Utc;
use reqwest::Client;

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
}
