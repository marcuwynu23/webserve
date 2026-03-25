use bytes::Bytes;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::RwLock;
use structopt::StructOpt;
use tokio::sync::broadcast;

pub type HtmlCache = Arc<RwLock<HashMap<PathBuf, Bytes>>>;
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
    /// When `--watch`: cache of path → injected HTML body; cleared when watcher fires.
    pub html_cache: Option<HtmlCache>,
}

/// Entry for one file or directory in a listing.
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<std::time::SystemTime>,
}

/// Why the chosen static root cannot be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticDirError {
    /// Path does not exist on disk.
    NotFound,
    /// Path exists but is not a directory (e.g. a file).
    NotADirectory,
}

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
