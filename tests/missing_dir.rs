//! Ensures a missing --dir produces a clear error (no silent bind).

use std::process::Command;

#[test]
fn missing_dir_exits_nonzero_and_stderr_helpful() {
    let missing = std::env::temp_dir().join("webserve_nonexistent_dir_99999_xyz");
    let _ = std::fs::remove_dir_all(&missing); // ensure missing

    let output = Command::new(env!("CARGO_BIN_EXE_webserve"))
        .args(["--dir", missing.to_str().unwrap()])
        .output()
        .expect("run webserve binary");

    assert!(
        !output.status.success(),
        "expected failure when directory does not exist"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let want = format!("{} not found", missing.display());
    assert!(
        stderr.contains(&want) || stderr.trim().ends_with("not found"),
        "stderr should be '<path> not found': {:?}",
        stderr
    );
}

#[test]
fn validate_static_root_unit() {
    use std::fs;
    use tempfile::TempDir;
    use webserve::{validate_static_root, StaticDirError};

    let temp = TempDir::new().unwrap();
    assert!(validate_static_root(temp.path()).is_ok());

    let missing = temp.path().join("nope_not_here");
    assert_eq!(validate_static_root(&missing), Err(StaticDirError::NotFound));

    let file_path = temp.path().join("file.txt");
    fs::write(&file_path, b"x").unwrap();
    assert_eq!(
        validate_static_root(&file_path),
        Err(StaticDirError::NotADirectory)
    );
}
