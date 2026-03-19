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
fn join_serve_path_rejects_traversal() {
    use std::path::Path;
    use tempfile::TempDir;
    use webserve::join_serve_path;

    let temp = TempDir::new().unwrap();
    let base = temp.path();
    assert!(join_serve_path(base, "..").is_none());
    assert!(join_serve_path(base, "a/../../etc/passwd").is_none());
    assert!(join_serve_path(base, "../outside").is_none());
    let j = join_serve_path(base, "/foo/bar").unwrap();
    assert_eq!(j, base.join("foo").join("bar"));
    assert_eq!(join_serve_path(base, "/").unwrap(), Path::new(base));
}

#[actix_web::test]
async fn serve_file_rejects_parent_dir() {
    use actix_web::{test, web, App as ActixApp};
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::broadcast;
    use webserve::{serve_file, AppState};

    let temp_dir = TempDir::new().unwrap();
    let static_dir = Arc::new(temp_dir.path().to_path_buf());
    let (tx, _) = broadcast::channel::<()>(16);
    let app_state = web::Data::new(AppState {
        static_dir,
        watch: false,
        spa: false,
        addr: "127.0.0.1:8080".to_string(),
        tx,
        redirect_dir_slash: true,
        reload_pending: Arc::new(AtomicBool::new(false)),
        html_cache: None,
    });
    let app = ActixApp::new()
        .app_data(app_state)
        .route("/{_:.*}", web::get().to(serve_file));
    let mut app = test::init_service(app).await;
    let req = test::TestRequest::get()
        .uri("/../Cargo.toml")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
}

#[test]
fn normalize_url_path_collapses_slashes() {
    use webserve::normalize_url_path;
    assert_eq!(normalize_url_path("//a///b//").as_deref(), Some("/a/b"));
    assert_eq!(normalize_url_path("/").as_deref(), Some("/"));
    assert!(normalize_url_path("/../x").is_none());
}

#[actix_web::test]
async fn serve_file_redirects_directory_without_slash() {
    use actix_web::{test, web, App as ActixApp};
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::broadcast;
    use webserve::{serve_file, AppState};

    let temp_dir = TempDir::new().unwrap();
    fs::create_dir(temp_dir.path().join("docs")).unwrap();
    fs::write(temp_dir.path().join("docs").join("index.html"), b"<p>hi</p>").unwrap();
    let static_dir = Arc::new(temp_dir.path().to_path_buf());
    let (tx, _) = broadcast::channel::<()>(16);
    let app_state = web::Data::new(AppState {
        static_dir,
        watch: false,
        spa: false,
        addr: "127.0.0.1:8080".to_string(),
        tx,
        redirect_dir_slash: true,
        reload_pending: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        html_cache: None,
    });
    let app = ActixApp::new()
        .app_data(app_state)
        .route("/{_:.*}", web::get().to(serve_file));
    let mut app = test::init_service(app).await;
    let req = test::TestRequest::get().uri("/docs").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::TEMPORARY_REDIRECT
    );
    assert_eq!(
        resp.headers().get("location").unwrap().to_str().unwrap(),
        "/docs/"
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
