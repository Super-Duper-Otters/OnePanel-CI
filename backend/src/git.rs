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

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub message: String,
    pub date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct FileStatus {
    pub path: String,
    pub status: String,
}

pub fn get_commit_log(path: &str, limit: usize) -> Result<Vec<CommitInfo>, String> {
    let repo = Repository::open(path).map_err(|e| e.to_string())?;
    let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;

    revwalk
        .push_head()
        .map_err(|_| "No head found".to_string())?;
    revwalk
        .set_sorting(git2::Sort::TIME)
        .map_err(|e| e.to_string())?;

    let mut commits = Vec::new();
    for oid in revwalk.take(limit) {
        let oid = oid.map_err(|e| e.to_string())?;
        let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;

        let time_seconds = commit.time().seconds();
        let datetime = Utc.timestamp_opt(time_seconds, 0).single();

        commits.push(CommitInfo {
            hash: oid.to_string(),
            author: commit.author().name().unwrap_or("Unknown").to_string(),
            message: commit.message().unwrap_or("").to_string(),
            date: datetime,
        });
    }

    Ok(commits)
}

pub fn get_detailed_status(path: &str) -> Result<Vec<FileStatus>, String> {
    let repo = Repository::open(path).map_err(|e| e.to_string())?;
    let statuses = repo.statuses(None).map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for entry in statuses.iter() {
        let status = entry.status();
        let status_str = if status.is_index_new() || status.is_wt_new() {
            "New"
        } else if status.is_index_modified() || status.is_wt_modified() {
            "Modified"
        } else if status.is_index_deleted() || status.is_wt_deleted() {
            "Deleted"
        } else if status.is_index_renamed() || status.is_wt_renamed() {
            "Renamed"
        } else if status.is_conflicted() {
            "Conflicted"
        } else {
            "Unknown"
        };

        if let Some(path) = entry.path() {
            result.push(FileStatus {
                path: path.to_string(),
                status: status_str.to_string(),
            });
        }
    }

    Ok(result)
}
