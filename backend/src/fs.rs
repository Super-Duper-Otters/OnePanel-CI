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

use git2::Repository;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ListRequest {
    pub path: Option<String>,
    pub root: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ScanRequest {
    pub path: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ReadFileRequest {
    pub path: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct WriteFileRequest {
    pub path: String,
    pub content: String,
}

pub fn read_file(path_str: &str) -> Result<String, String> {
    let path = PathBuf::from(path_str);
    if !path.exists() {
        return Err("File does not exist".to_string());
    }
    fs::read_to_string(path).map_err(|e| e.to_string())
}

pub fn list_directory(
    path_str: Option<String>,
    root_str: Option<String>,
) -> Result<Vec<FileEntry>, String> {
    let path = match path_str {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().map_err(|e| e.to_string())?,
    };

    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    // Security check: if root is provided, path must be inside root
    if let Some(ref root) = root_str {
        let root_path = PathBuf::from(root);
        if !path.starts_with(&root_path) {
            return Err("Access denied: Path is outside the root directory".to_string());
        }
    }

    let mut entries = Vec::new();

    // Add parent directory ".." only if we are not at the root
    let is_at_root = if let Some(ref root) = root_str {
        PathBuf::from(root) == path
    } else {
        false
    };

    if !is_at_root {
        if let Some(parent) = path.parent() {
            entries.push(FileEntry {
                name: "..".to_string(),
                path: parent.to_string_lossy().to_string(),
                is_dir: true,
            });
        }
    }

    // Attempt to open git repo for ignore checking
    let repo = if let Some(ref root) = root_str {
        Repository::open(root).ok()
    } else {
        Repository::discover(&path).ok()
    };

    if let Ok(read_dir) = fs::read_dir(&path) {
        for entry in read_dir.flatten() {
            let metadata = entry.metadata().ok();
            let is_dir = metadata.map(|m| m.is_dir()).unwrap_or(false);
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Check if ignored
            // NOTE: .git folder itself is usually not "ignored" by is_path_ignored but we should hide it?
            // User request implies hiding standard ignored files.
            if name == ".git" {
                continue;
            }

            if let Some(ref repo) = repo {
                // is_path_ignored expects path relative to repo root
                if let Some(repo_workdir) = repo.workdir() {
                    if let Ok(relative_path) = entry_path.strip_prefix(repo_workdir) {
                        if let Ok(true) = repo.is_path_ignored(relative_path) {
                            continue;
                        }
                    }
                }
            }

            entries.push(FileEntry {
                name,
                path: entry_path.to_string_lossy().to_string(),
                is_dir,
            });
        }
    }

    // Sort: Directories first, then files
    entries.sort_by(|a, b| {
        if a.name == ".." {
            return std::cmp::Ordering::Less;
        }
        if b.name == ".." {
            return std::cmp::Ordering::Greater;
        }
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
