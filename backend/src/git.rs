use chrono::{DateTime, TimeZone, Utc};
use git2::Repository;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct GitStatus {
    pub path: String,
    pub branch: Option<String>,
    pub last_commit_message: Option<String>,
    pub last_commit_time: Option<DateTime<Utc>>,
    pub is_clean: bool,
}

pub fn get_repo_status(path: &str) -> Result<GitStatus, String> {
    let repo = Repository::open(path).map_err(|e| e.to_string())?;

    // Get branch name
    let head = repo.head().ok();
    let branch = head
        .as_ref()
        .and_then(|h| h.shorthand())
        .map(|s| s.to_string());

    // Get last commit info
    let (message, time) = if let Some(h) = head {
        if let Ok(commit) = h.peel_to_commit() {
            let msg = commit.message().unwrap_or("").to_string();
            let time_seconds = commit.time().seconds();
            let datetime = Utc.timestamp_opt(time_seconds, 0).single();
            (Some(msg), datetime)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // Check if clean (simplified check for modified files)
    let statuses = repo.statuses(None).map_err(|e| e.to_string())?;
    let is_clean = statuses.is_empty();

    Ok(GitStatus {
        path: path.to_string(),
        branch,
        last_commit_message: message,
        last_commit_time: time,
        is_clean,
    })
}
