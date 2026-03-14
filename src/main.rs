//! Binary entry point for webserve

use actix_web::{web, App, HttpServer};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use tokio::sync::broadcast;
use webserve::{
    reload_poll, serve_file, validate_static_root, AppState, ServeOptions, StaticDirError,
};

fn log_info(msg: &str) {
    println!("[INFO] {}", msg);
}

/// Exit when `--dir` (or the resolved root) is invalid.
fn fail_static_dir(path: &Path, err: StaticDirError) -> ! {
    match err {
        StaticDirError::NotFound => eprintln!("{} not found", path.display()),
        StaticDirError::NotADirectory => eprintln!("{} is not a directory", path.display()),
    }
    std::process::exit(1);
}

fn listen_error(addr: &str, e: &io::Error) -> String {
    match e.kind() {
        io::ErrorKind::AddrInUse => format!("{} already in use", addr),
        io::ErrorKind::PermissionDenied => format!("permission denied binding to {}", addr),
        io::ErrorKind::AddrNotAvailable => format!("address not available: {}", addr),
        io::ErrorKind::InvalidInput => format!("invalid listen address {}", addr),
        _ => format!("cannot listen on {}: {}", addr, e),
    }
}

#[actix_web::main]
async fn main() {
    if let Err(msg) = run().await {
        eprintln!("{}", msg);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let options = ServeOptions::from_args();
    let static_dir = Arc::new(if let Some(ref p) = options.directory {
        p.clone()
    } else {
        std::env::current_dir().map_err(|e| format!("working directory unavailable: {}", e))?
    });

    if let Err(e) = validate_static_root(&static_dir) {
        fail_static_dir(&static_dir, e);
    }

    let addr = format!("{}:{}", options.host, options.port);
    let (tx, _rx) = broadcast::channel::<()>(16);

    log_info("Starting webserve");
    log_info(&format!("Directory: {}", static_dir.display()));
    log_info(&format!("Host: {}", options.host));
    log_info(&format!("Port: {}", options.port));
    if options.spa {
        log_info("SPA mode: enabled");
    }
    if options.watch {
        log_info("Watch: enabled");
    }

    let app_state = web::Data::new(AppState {
        static_dir: static_dir.clone(),
        watch: options.watch,
        spa: options.spa,
        addr: addr.clone(),
        tx: tx.clone(),
    });

    if options.watch {
        let watch_path = static_dir.clone();
        let tx_watcher = tx.clone();
        let mut watcher: RecommendedWatcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                if res.is_ok() {
                    let _ = tx_watcher.send(());
                }
            })
        .map_err(|e| format!("file watch unavailable: {}", e))?;
        watcher
            .watch(&watch_path, RecursiveMode::Recursive)
            .map_err(|e| format!("cannot watch {}: {}", watch_path.display(), e))?;
        thread::spawn(move || {
            let _keep_alive = watcher;
            loop {
                thread::sleep(Duration::from_secs(60));
            }
        });
        log_info(&format!("Watching directory: {}", watch_path.display()));
    }

    log_info(&format!("Serving on http://{}", addr));

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/reload", web::get().to(reload_poll))
            .route("/{_:.*}", web::get().to(serve_file))
    })
    .bind(&addr)
    .map_err(|e| listen_error(&addr, &e))?
    .run()
    .await
    .map_err(|e| format!("server error: {}", e))?;

    Ok(())
}
