//! Integration tests for webserve

use actix_web::{test, web, App as ActixApp};
use std::fs;
use std::io::Write;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::broadcast;
use webserve::{directory_listing, reload_poll, serve_file, AppState};

#[tokio::test]
async fn test_directory_listing_empty() {
    let temp_dir = TempDir::new().unwrap();
    let listing = directory_listing(temp_dir.path()).await;
    assert!(listing.contains("<ul>"));
    assert!(listing.contains("</ul>"));
}

#[tokio::test]
async fn test_directory_listing_with_files() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::File::create(&file_path).unwrap();

    let listing = directory_listing(temp_dir.path()).await;
    assert!(listing.contains("test.txt"));
    assert!(listing.contains("<a href="));
}

#[actix_web::test]
async fn test_serve_file_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let mut file = fs::File::create(&file_path).unwrap();
    writeln!(file, "Hello, World!").unwrap();
    drop(file);

    let static_dir = Arc::new(temp_dir.path().to_path_buf());
    let (tx, _) = broadcast::channel::<()>(16);
    let app_state = web::Data::new(AppState {
        static_dir,
        watch: false,
        spa: false,
        addr: "127.0.0.1:8080".to_string(),
        tx,
    });

    let app = ActixApp::new()
        .app_data(app_state.clone())
        .route("/{_:.*}", web::get().to(serve_file));

    let mut app = test::init_service(app).await;
    let req = test::TestRequest::get().uri("/test.txt").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_serve_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let static_dir = Arc::new(temp_dir.path().to_path_buf());
    let (tx, _) = broadcast::channel::<()>(16);
    let app_state = web::Data::new(AppState {
        static_dir,
        watch: false,
        spa: false,
        addr: "127.0.0.1:8080".to_string(),
        tx,
    });

    let app = ActixApp::new()
        .app_data(app_state.clone())
        .route("/{_:.*}", web::get().to(serve_file));

    let mut app = test::init_service(app).await;
    let req = test::TestRequest::get()
        .uri("/nonexistent.txt")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_serve_file_spa_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("index.html");
    let mut file = fs::File::create(&index_path).unwrap();
    writeln!(file, "<html><body>SPA</body></html>").unwrap();
    drop(file);

    let static_dir = Arc::new(temp_dir.path().to_path_buf());
    let (tx, _) = broadcast::channel::<()>(16);
    let app_state = web::Data::new(AppState {
        static_dir,
        watch: false,
        spa: true,
        addr: "127.0.0.1:8080".to_string(),
        tx,
    });

    let app = ActixApp::new()
        .app_data(app_state.clone())
        .route("/{_:.*}", web::get().to(serve_file));

    let mut app = test::init_service(app).await;
    let req = test::TestRequest::get()
        .uri("/nonexistent-route")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_serve_file_directory_listing() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::File::create(&file_path).unwrap();

    let static_dir = Arc::new(temp_dir.path().to_path_buf());
    let (tx, _) = broadcast::channel::<()>(16);
    let app_state = web::Data::new(AppState {
        static_dir,
        watch: false,
        spa: false,
        addr: "127.0.0.1:8080".to_string(),
        tx,
    });

    let app = ActixApp::new()
        .app_data(app_state.clone())
        .route("/{_:.*}", web::get().to(serve_file));

    let mut app = test::init_service(app).await;
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("test.txt"));
    assert!(body_str.contains("<ul>"));
}

#[actix_web::test]
async fn test_reload_poll() {
    let temp_dir = TempDir::new().unwrap();
    let static_dir = Arc::new(temp_dir.path().to_path_buf());
    let (tx, _) = broadcast::channel::<()>(16);
    let app_state = web::Data::new(AppState {
        static_dir,
        watch: false,
        spa: false,
        addr: "127.0.0.1:8080".to_string(),
        tx: tx.clone(),
    });

    let app = ActixApp::new()
        .app_data(app_state.clone())
        .route("/reload", web::get().to(reload_poll));

    let mut app = test::init_service(app).await;

    // Send a reload signal after the service is initialized
    // Use tokio::spawn to send it asynchronously so the handler can receive it
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let _ = tx_clone.send(());
    });

    let req = test::TestRequest::get().uri("/reload").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    assert_eq!(body, "reload");
}
