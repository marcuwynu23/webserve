//! Command-line interface and options.

use std::path::PathBuf;
use structopt::StructOpt;

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
