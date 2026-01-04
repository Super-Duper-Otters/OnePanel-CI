use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use utoipa::ToSchema;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ListRequest {
    pub path: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ScanRequest {
    pub path: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ReadFileRequest {
    pub path: String,
}

pub fn read_file(path_str: &str) -> Result<String, String> {
    let path = PathBuf::from(path_str);
    if !path.exists() {
        return Err("File does not exist".to_string());
    }
    fs::read_to_string(path).map_err(|e| e.to_string())
}

pub fn list_directory(path_str: Option<String>) -> Result<Vec<FileEntry>, String> {
    let path = match path_str {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().map_err(|e| e.to_string())?,
    };

    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    let mut entries = Vec::new();

    // Add parent directory ".."
    if let Some(parent) = path.parent() {
        entries.push(FileEntry {
            name: "..".to_string(),
            path: parent.to_string_lossy().to_string(),
            is_dir: true,
        });
    }

    if let Ok(read_dir) = fs::read_dir(&path) {
        for entry in read_dir.flatten() {
            let metadata = entry.metadata().ok();
            let is_dir = metadata.map(|m| m.is_dir()).unwrap_or(false);

            entries.push(FileEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                is_dir,
            });
        }
    }

    // Sort: Directories first, then files
    entries.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.name.cmp(&b.name)
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Ok(entries)
}

pub fn scan_for_git_repos(root: &str) -> Vec<String> {
    let mut repos = Vec::new();

    // Use WalkDir to recursively find .git directories
    // We filter for directories named ".git"
    // and return their parent path
    for entry in WalkDir::new(root)
        .min_depth(1)
        .max_depth(5) // Limit depth to prevent hanging on deep structures
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() && entry.file_name() == ".git" {
            if let Some(parent) = entry.path().parent() {
                repos.push(parent.to_string_lossy().to_string());
            }
        }
    }

    repos
}
