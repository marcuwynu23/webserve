//! Path normalization and static root validation.

use std::path::{Component, Path, PathBuf};

/// Why the chosen static root cannot be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticDirError {
    /// Path does not exist on disk.
    NotFound,
    /// Path exists but is not a directory (e.g. a file).
    NotADirectory,
}

/// Ensures the server root exists and is a directory before binding.
pub fn validate_static_root(path: &Path) -> Result<(), StaticDirError> {
    if !path.exists() {
        Err(StaticDirError::NotFound)
    } else if !path.is_dir() {
        Err(StaticDirError::NotADirectory)
    } else {
        Ok(())
    }
}

/// Collapses repeated `/` and `.` segments; rejects `..`. Root is `/`.
pub fn normalize_url_path(path: &str) -> Option<String> {
    let mut out: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            return None;
        }
        out.push(seg);
    }
    if out.is_empty() {
        Some("/".to_string())
    } else {
        Some(format!("/{}", out.join("/")))
    }
}

/// Joins normalized URL path onto the serve root (no `..`).
pub fn join_serve_path(base: &Path, normalized_path: &str) -> Option<PathBuf> {
    let rel = normalized_path.trim_start_matches('/');
    let mut out = base.to_path_buf();
    for c in Path::new(rel).components() {
        match c {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(out)
}
