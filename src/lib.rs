//! # Webserve
//!
//! A lightweight static file server built with Actix-Web,
//! featuring optional directory listing, SPA fallback routing,
//! and live-reload functionality using filesystem watchers.
//!
//! ## Features
//! - Serves static files from a directory
//! - Single Page Application (SPA) mode (fallback to `index.html`)
//! - Directory listing if no `index.html` is found
//! - Optional file watcher for live reloads via polling
//! - Customizable host and port
//!
//! ## Example
//! ```bash
//! webserve --dir ./public --port 3000 --watch --spa
//! ```

use actix_files::NamedFile;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use structopt::StructOpt;
use tokio::sync::broadcast;

/// Command-line options parsed via StructOpt.
#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "webserve",
    about = "A simple static file server with live reload."
)]
pub struct ServeOptions {
    /// The port to listen on (default: 8080)
    #[structopt(short = "p", long = "port", default_value = "8080")]
    pub port: u16,

    /// The host address to bind to (default: 127.0.0.1)
    #[structopt(short = "h", long = "host", default_value = "127.0.0.1")]
    pub host: String,

    /// The directory to serve files from (defaults to current directory)
    #[structopt(short = "d", long = "dir", parse(from_os_str))]
    pub directory: Option<PathBuf>,

    /// Enable Single Page Application (SPA) mode — fall back to index.html
    #[structopt(long = "spa")]
    pub spa: bool,

    /// Enable live reload by watching for file changes
    #[structopt(short = "w", long = "watch")]
    pub watch: bool,
}

/// Shared application state accessible by Actix handlers.
pub struct AppState {
    pub static_dir: Arc<PathBuf>,
    pub watch: bool,
    pub spa: bool,
    pub addr: String,
    pub tx: broadcast::Sender<()>,
}

/// Generates a simple HTML directory listing for the given path.
///
/// Each entry is a hyperlink to the file or subdirectory.
pub async fn directory_listing(path: &Path) -> String {
    let mut listing = String::from("<ul>");
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            listing.push_str(&format!(
                "<li><a href=\"{}\" style=\"text-decoration:none; font-size:1.1em; display:block;\">{}</a></li>",
                name, name
            ));
        }
    }
    listing.push_str("</ul>");
    listing
}

/// Handles file requests.
///
/// - Serves static files from the given directory.
/// - Provides directory listings if no `index.html` exists.
/// - Falls back to `index.html` if in SPA mode.
/// - Optionally injects a live reload script when `--watch` is enabled.
pub async fn serve_file(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let base_dir = &data.static_dir;
    let path = req.path().trim_start_matches('/');
    let mut file_path = base_dir.join(path);

    // If the request points to a directory, check for an index.html file
    if file_path.is_dir() {
        let index_file = file_path.join("index.html");
        if index_file.exists() {
            file_path = index_file;
        } else {
            let listing = directory_listing(&file_path).await;
            return Ok(HttpResponse::Ok().content_type("text/html").body(listing));
        }
    }

    // SPA fallback: return index.html if file not found
    if !file_path.exists() && data.spa {
        let spa_index = base_dir.join("index.html");
        if spa_index.exists() {
            file_path = spa_index;
        } else {
            return Ok(HttpResponse::NotFound().finish());
        }
    } else if !file_path.exists() {
        return Ok(HttpResponse::NotFound().finish());
    }

    // Serve file
    let named_file = NamedFile::open_async(file_path).await?;

    // Inject live reload script into HTML if watch mode is on
    if data.watch {
        if let Some(ext) = named_file.path().extension() {
            if ext == "html" {
                let addr = &data.addr;
                let ws_script = format!(
                    r#"<script>
    async function checkReload() {{
        try {{
            const res = await fetch("http://{}/reload");
            if(res.ok) {{
                location.reload();
            }}
        }} catch(e) {{
            console.error(e);
        }}
        setTimeout(checkReload, 1000);
    }}
    checkReload();
    </script>"#,
                    addr
                );

                let mut body = tokio::fs::read(named_file.path()).await?;
                body.extend(ws_script.as_bytes());
                return Ok(HttpResponse::Ok().content_type("text/html").body(body));
            }
        }
    }

    Ok(named_file.into_response(&req))
}

/// Endpoint that clients poll to detect file changes.
///
/// When a change is detected by the file watcher,
/// this endpoint returns an HTTP 200 response prompting the client to reload.
pub async fn reload_poll(data: web::Data<AppState>) -> impl Responder {
    let mut rx = data.tx.subscribe();
    let _ = rx.recv().await; // Wait for broadcast event
    HttpResponse::Ok().body("reload")
}
