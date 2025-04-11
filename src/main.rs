use futures_util::SinkExt;
use mime_guess::from_path;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use warp::http::{Response, StatusCode, header::CONTENT_TYPE};
use warp::{Filter, ws::Message};

#[derive(StructOpt, Debug)]
#[structopt(name = "myserve")]
struct ServeOptions {
    #[structopt(short = "p", long = "port", default_value = "8080")]
    port: u16,

    #[structopt(short = "h", long = "host", default_value = "127.0.0.1")]
    host: String,

    #[structopt(short = "d", long = "dir", parse(from_os_str))]
    directory: Option<PathBuf>,

    #[structopt(long = "spa")]
    spa: bool,

    #[structopt(short = "w", long = "watch")]
    watch: bool,
}

#[tokio::main]
async fn main() {
    let options = Arc::new(ServeOptions::from_args());

    let addr: SocketAddr = format!("{}:{}", options.host, options.port)
        .parse()
        .expect("Invalid host or port");
    let static_dir = Arc::new(
        options
            .directory
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap()),
    );
    let spa_enabled = options.spa;
    let watch_enabled = options.watch;
    let (reload_tx, _) = tokio::sync::broadcast::channel::<()>(16);

    if watch_enabled {
        let watch_path = static_dir.clone();
        let tx = reload_tx.clone();
        thread::spawn(move || {
            let (_watch_tx, _watch_rx) = channel::<notify::Result<notify::Event>>();
            let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| {
                if let Ok(event) = res {
                    println!("🔁 File change detected: {:?}", event);
                    let _ = tx.send(());
                }
            })
            .expect("Failed to create watcher");
            watcher
                .watch(&watch_path, RecursiveMode::Recursive)
                .expect("Failed to watch directory");
            println!("👀 Watching directory for changes: {:?}", watch_path);
            loop {
                std::thread::sleep(Duration::from_secs(60));
            }
        });
    }

    let reload_ws = warp::path("reload")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let mut rx = reload_tx.subscribe();
            ws.on_upgrade(move |mut socket| async move {
                while rx.recv().await.is_ok() {
                    if socket.send(Message::text("reload")).await.is_err() {
                        break;
                    }
                }
            })
        });

    let static_files = {
        let static_dir = Arc::clone(&static_dir);
        warp::path::full()
            .and(warp::get())
            .and_then(move |full_path: warp::path::FullPath| {
                let static_dir = Arc::clone(&static_dir);
                let spa_enabled = spa_enabled;
                let watch_enabled = watch_enabled;
                async move {
                    let req_path = full_path.as_str();
                    let mut file_path = static_dir.join(req_path.trim_start_matches("/"));

                    let metadata = fs::metadata(&file_path).await.ok();
                    if metadata.map(|m| m.is_dir()).unwrap_or(false) || req_path == "/" {
                        file_path = file_path.join("index.html");
                    }

                    if spa_enabled && fs::metadata(&file_path).await.is_err() {
                        file_path = static_dir.join("index.html");
                    }

                    let mut buffer = Vec::new();
                    if let Ok(mut file) = File::open(&file_path).await {
                        file.read_to_end(&mut buffer).await.unwrap();
                    } else {
                        return Ok::<_, Infallible>(Response::builder().status(StatusCode::NOT_FOUND).body(Vec::new()).unwrap());
                    }

                    let mime_type = from_path(&file_path).first_or_octet_stream();
                    let response = if watch_enabled && file_path.ends_with("index.html") {
                        if let Ok(mut content) = String::from_utf8(buffer.clone()) {
                            content.push_str("<script>const socket = new WebSocket(`ws://${location.host}/reload`); socket.onmessage = () => location.reload();</script>");
                            Response::builder()
                                .header(CONTENT_TYPE, "text/html")
                                .body(content.into_bytes())
                                .unwrap()
                        } else {
                            Response::builder()
                                .header(CONTENT_TYPE, mime_type.as_ref())
                                .body(buffer)
                                .unwrap()
                        }
                    } else {
                        Response::builder()
                            .header(CONTENT_TYPE, mime_type.as_ref())
                            .body(buffer)
                            .unwrap()
                    };

                    Ok::<_, Infallible>(response)
                }
            })
    };

    let routes = reload_ws.or(static_files);

    println!("\n🚀 Serving on http://{addr}");
    println!("📁 Directory: {:?}", static_dir);
    if spa_enabled {
        println!("🔁 SPA mode: Enabled");
    }
    if watch_enabled {
        println!("👀 Watch mode is enabled");
        println!("🔄 Live reload via ws://{}/reload", addr);
    }

    warp::serve(routes).run(addr).await;
}
