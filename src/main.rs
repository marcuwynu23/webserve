use futures::SinkExt;

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
use tokio::net::TcpListener;

use tokio_stream::wrappers::TcpListenerStream;

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

async fn list_directory_contents(file_path: &std::path::Path) -> String {
    let mut dir_list = String::new();
    let mut dir_entries = fs::read_dir(file_path).await.unwrap(); // ReadDir as a stream

    dir_list.push_str("<h4>Directory listing for </h4><ul>");

    // Asynchronously iterate over the directory entries using next()
    while let Some(entry) = dir_entries.next_entry().await.unwrap() {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();
        dir_list.push_str(&format!("<li><a href=\"{}\" style=\"text-decoration: none; font-size: 1.1em; display: block;\">{}</a></li>", name, name));
    }

    dir_list.push_str("</ul>");
    dir_list
}

#[tokio::main]
async fn main() {
    let options = ServeOptions::from_args();

    // Show help if no arguments are provided
    if options.host.is_empty() && options.port == 8080 && options.directory.is_none() {
        ServeOptions::clap().print_long_help().unwrap();
        return;
    }

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
                if let Ok(_event) = res {
                    let _ = tx.send(()); // Trigger reload when file changes
                }
            })
            .expect("Failed to create watcher");
            watcher
                .watch(&watch_path, RecursiveMode::Recursive)
                .expect("Failed to watch directory");
            println!("Watching directory for changes: {:?}", watch_path);
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
                // let spa_enabled = spa_enabled;
                async move {
                    let req_path = full_path.as_str();
                    let mut file_path = static_dir.join(req_path.trim_start_matches("/"));

                    // Check if the requested path is a directory
                    let metadata = fs::metadata(&file_path).await.ok();
                    if metadata.map(|m| m.is_dir()).unwrap_or(false) {
                        let index_path = static_dir.join("index.html");

                        if index_path.exists() {
                            // Use index.html if it exists
                            file_path = index_path;
                        } else {
                            // Return directory listing if index.html is not found
                            let dir_listing = list_directory_contents(&file_path).await;
                            return Ok::<_, Infallible>(
                                Response::builder()
                                    .status(StatusCode::OK)
                                    .header(CONTENT_TYPE, "text/html")
                                    .body(dir_listing.into_bytes())
                                    .unwrap(),
                            );
                        }
                    }

                    // Handle files (non-directory)
                    // Handle files (non-directory)
                    let mut buffer = Vec::new();
                    if let Ok(mut file) = File::open(&file_path).await {
                        file.read_to_end(&mut buffer).await.unwrap();
                    } else {
                        return Ok::<_, Infallible>(
                            Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Vec::new())
                                .unwrap(),
                        );
                    }

                    let mime_type = from_path(&file_path).first_or_octet_stream();
                    let content_type = if file_path.extension().unwrap_or_default() == "js" {
                        "application/javascript"
                    } else {
                        mime_type.as_ref()
                    };

                    // Inject live reload script into HTML pages (if watch mode is on)
                    let mut body = buffer;
                    let is_html = file_path
                        .extension()
                        .map(|ext| ext == "html")
                        .unwrap_or(false);

                    if is_html && watch_enabled {
                        let reload_script = format!(
                            r#"<script>
            const ws = new WebSocket("ws://{}/reload");
            ws.onmessage = (ev) => {{
                if (ev.data === "reload") {{
                    console.log("Live reload triggered");
                    location.reload();
                }}
            }};
            ws.onclose = () => console.warn("Live reload connection closed");
        </script>"#,
                            addr
                        );

                        let mut html = String::from_utf8_lossy(&body).to_string();
                        html.push_str(&reload_script);
                        body = html.into_bytes();
                    }

                    let response = Response::builder()
                        .header(CONTENT_TYPE, content_type)
                        .body(body)
                        .unwrap();

                    Ok::<_, Infallible>(response)
                }
            })
    };

    let routes = reload_ws.or(static_files);

    println!("\nServing on http://{addr}");
    println!("Directory: {:?}", static_dir);
    if spa_enabled {
        println!("SPA mode: Enabled");
    }
    if watch_enabled {
        println!("Watch mode is enabled");
        println!("Live reload via ws://{}/reload", addr);
    }

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Could not bind to {addr}: {e}");
            std::process::exit(1);
        }
    };

    let incoming = TcpListenerStream::new(listener);
    warp::serve(routes).run_incoming(incoming).await;
}
