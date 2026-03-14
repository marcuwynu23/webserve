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
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
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

    /// Open the default browser to the server URL after startup
    #[structopt(long = "open")]
    pub open: bool,

    /// Do not redirect to add a trailing slash when the URL names a directory (default: redirect)
    #[structopt(long = "no-redirect-dir-slash")]
    pub no_redirect_dir_slash: bool,
}

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

/// Shared application state accessible by Actix handlers.
pub struct AppState {
    pub static_dir: Arc<PathBuf>,
    pub watch: bool,
    pub spa: bool,
    pub addr: String,
    pub tx: broadcast::Sender<()>,
    /// Redirect GET when URL names a directory but has no trailing `/`.
    pub redirect_dir_slash: bool,
    /// Set by filesystem watcher; `/reload` clears and tells clients to refresh.
    pub reload_pending: Arc<AtomicBool>,
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
    let Some(canonical_path) = normalize_url_path(req.path()) else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let Some(mut file_path) = join_serve_path(base_dir, &canonical_path) else {
        return Ok(HttpResponse::NotFound().finish());
    };

    // Directory without trailing slash -> redirect to .../ (normalized URLs always lack trailing slash except root)
    if data.redirect_dir_slash
        && file_path.is_dir()
        && canonical_path != "/"
        && !req.path().ends_with('/')
    {
        let location = if canonical_path == "/" {
            "/".to_string()
        } else {
            format!("{}/", canonical_path)
        };
        let mut r = HttpResponse::build(actix_web::http::StatusCode::TEMPORARY_REDIRECT);
        if let Some(q) = req.uri().query() {
            r.insert_header((actix_web::http::header::LOCATION, format!("{}?{}", location, q)));
        } else {
            r.insert_header((actix_web::http::header::LOCATION, location));
        }
        return Ok(r.finish());
    }

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

    // Serve file (race: gone after exists check → 404)
    let named_file = match NamedFile::open_async(&file_path).await {
        Ok(f) => f,
        Err(_) => return Ok(HttpResponse::NotFound().finish()),
    };

    // Inject live reload script into HTML if watch mode is on
    if data.watch {
        if let Some(ext) = named_file.path().extension() {
            if ext == "html" {
                // Relative /reload + short polling (no long-lived pending requests)
                let ws_script = r#"<script>
(function(){
  async function tick(){
    try {
      var r = await fetch("/reload", { cache: "no-store" });
      if (r.ok && r.status === 200) {
        var t = await r.text();
        if (t === "reload") { location.reload(); return; }
      }
    } catch(e) { console.error(e); }
    setTimeout(tick, 600);
  }
  tick();
})();
</script>"#
                    .to_string();

                let read_path = named_file.path().to_path_buf();
                let mut body = match tokio::fs::read(&read_path).await {
                    Ok(b) => b,
                    Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
                };
                body.extend(ws_script.as_bytes());
                return Ok(HttpResponse::Ok().content_type("text/html").body(body));
            }
        }
    }

    Ok(named_file.into_response(&req))
}

/// Short poll: 200 + body `reload` if a file changed since last poll; otherwise 204 immediately.
pub async fn reload_poll(data: web::Data<AppState>) -> impl Responder {
    if data
        .reload_pending
        .swap(false, Ordering::SeqCst)
    {
        HttpResponse::Ok()
            .content_type("text/plain")
            .body("reload")
    } else {
        HttpResponse::NoContent().finish()
    }
}
