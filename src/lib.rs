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

pub mod path;
pub mod serve;
pub mod types;

pub use path::{join_serve_path, normalize_url_path, validate_static_root};
pub use serve::{directory_listing, reload_poll, serve_file};
pub use types::{AppState, DirEntry, ServeOptions, StaticDirError};
