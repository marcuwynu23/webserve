//! Binary entry point for webserve

use actix_web::{web, App, HttpServer};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use tokio::sync::broadcast;
use webserve::{reload_poll, serve_file, AppState, ServeOptions};

/// Application entry point.
///
/// Initializes the server, file watcher (if enabled),
/// and starts listening for incoming HTTP connections.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let options = ServeOptions::from_args();
    let static_dir = Arc::new(
        options
            .directory
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap()),
    );

    let addr = format!("{}:{}", options.host, options.port);
    let (tx, _rx) = broadcast::channel::<()>(16);

    let app_state = web::Data::new(AppState {
        static_dir: static_dir.clone(),
        watch: options.watch,
        spa: options.spa,
        addr: addr.clone(),
        tx: tx.clone(),
    });

    // Watcher thread: monitors the static directory for changes
    if options.watch {
        let watch_path = static_dir.clone();
        let tx_watcher = tx.clone();
        thread::spawn(move || {
            let (_tx, _rx) = channel::<notify::Result<notify::Event>>();
            let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| {
                if let Ok(_event) = res {
                    let _ = tx_watcher.send(()); // Broadcast reload signal
                }
            })
            .expect("Failed to create watcher");
            watcher
                .watch(&watch_path, RecursiveMode::Recursive)
                .expect("Failed to watch directory");
            println!("Watching directory: {:?}", watch_path);
            loop {
                thread::sleep(Duration::from_secs(60));
            }
        });
    }

    println!("Serving on http://{}", addr);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/reload", web::get().to(reload_poll))
            .route("/{_:.*}", web::get().to(serve_file))
    })
    .bind(addr)?
    .run()
    .await
}
