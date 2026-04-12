//! HTTP serving: app state, directory listing, file handler, reload poll.

use crate::path::{join_serve_path, normalize_url_path};
use actix_files::NamedFile;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use bytes::Bytes;
use std::path::Path;
use std::sync::atomic::Ordering;

use crate::{AppState, DirEntry};

/// Generates a full HTML page with a styled directory listing.
///
/// `url_prefix` is the current directory's URL path with trailing slash (e.g. `/` or `/foo/bar/`).
pub async fn directory_listing(path: &Path, url_prefix: &str) -> String {
    let mut dirs: Vec<DirEntry> = Vec::new();
    let mut files: Vec<DirEntry> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let meta = entry.metadata().ok();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = meta
                .as_ref()
                .and_then(|m| if m.is_file() { Some(m.len()) } else { None });
            let modified = meta.and_then(|m| m.modified().ok());

            let e = DirEntry {
                name,
                is_dir,
                size,
                modified,
            };
            if e.is_dir {
                dirs.push(e);
            } else {
                files.push(e);
            }
        }
    }

    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let breadcrumb = format_breadcrumb(url_prefix);
    let path_for_title = url_prefix.trim_end_matches('/');
    let title = if path_for_title.is_empty() || path_for_title == "/" {
        "Index of /".to_string()
    } else {
        format!("Index of {}", path_for_title)
    };

    let mut rows = String::new();
    let base = url_prefix.trim_end_matches('/');
    let base = if base.is_empty() { "/" } else { base };

    for e in dirs {
        let encoded = percent_encode_path_segment(&e.name);
        let href = if base == "/" {
            format!("/{}/", encoded)
        } else {
            format!("{}/{}/", base, encoded)
        };
        let size_str = String::from("—");
        let date_str = format_time(e.modified);
        rows.push_str(&format_entry_row(
            &e.name, &href, true, &size_str, &date_str,
        ));
    }
    for e in files {
        let encoded = percent_encode_path_segment(&e.name);
        let href = if base == "/" {
            format!("/{}", encoded)
        } else {
            format!("{}/{}", base, encoded)
        };
        let size_str = format_size(e.size.unwrap_or(0));
        let date_str = format_time(e.modified);
        rows.push_str(&format_entry_row(
            &e.name, &href, false, &size_str, &date_str,
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en" data-theme="light">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500&family=Outfit:wght@400;500;600&display=swap" rel="stylesheet">
  <style>
    :root, [data-theme="dark"] {{
      --bg: #0f0f12;
      --surface: #18181c;
      --border: #2a2a30;
      --text: #e4e4e7;
      --text-muted: #71717a;
      --accent: #a78bfa;
      --accent-hover: #c4b5fd;
      --dir-color: #fbbf24;
      --hover-bg: rgba(255,255,255,0.03);
    }}
    [data-theme="light"] {{
      --bg: #f4f4f5;
      --surface: #ffffff;
      --border: #e4e4e7;
      --text: #18181b;
      --text-muted: #71717a;
      --accent: #7c3aed;
      --accent-hover: #6d28d9;
      --dir-color: #b45309;
      --hover-bg: rgba(0,0,0,0.04);
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      min-height: 100vh;
      background: var(--bg);
      color: var(--text);
      font-family: 'Outfit', system-ui, sans-serif;
      font-size: 15px;
      line-height: 1.5;
    }}
    .container-fluid {{
      width: 100%;
      padding: 2rem 1.5rem;
    }}
    .header-row {{
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-content: space-between;
      gap: 1rem;
      margin-bottom: 1.5rem;
    }}
    .header-left {{ flex: 1; min-width: 0; }}
    h1 {{
      font-size: 1rem;
      font-weight: 600;
      color: var(--text-muted);
      margin: 0 0 0.25rem 0;
      letter-spacing: 0.02em;
    }}
    .breadcrumb {{
      font-size: 0.875rem;
      color: var(--text-muted);
    }}
    .breadcrumb a {{
      color: var(--accent);
      text-decoration: none;
    }}
    .breadcrumb a:hover {{ color: var(--accent-hover); }}
    .breadcrumb span {{ color: var(--text-muted); margin: 0 0.35rem; }}
    .theme-toggle {{
      flex-shrink: 0;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 2.25rem;
      height: 2.25rem;
      padding: 0;
      border: 1px solid var(--border);
      border-radius: 8px;
      background: var(--surface);
      color: var(--text);
      cursor: pointer;
    }}
    .theme-toggle:hover {{ background: var(--hover-bg); }}
    .theme-toggle .theme-icon {{ width: 1.2rem; height: 1.2rem; }}
    .theme-toggle .theme-icon-sun {{ display: none; }}
    .theme-toggle .theme-icon-moon {{ display: block; }}
    [data-theme="dark"] .theme-toggle .theme-icon-sun {{ display: block; }}
    [data-theme="dark"] .theme-toggle .theme-icon-moon {{ display: none; }}
    table {{
      width: 100%;
      border-collapse: collapse;
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 10px;
      overflow: hidden;
    }}
    th {{
      text-align: left;
      font-weight: 500;
      font-size: 0.75rem;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      color: var(--text-muted);
      padding: 0.65rem 1rem;
      border-bottom: 1px solid var(--border);
    }}
    th:first-child {{ padding-left: 1.25rem; }}
    th.size {{ width: 6rem; }}
    th.date {{ width: 10rem; }}
    td {{
      padding: 0.6rem 1rem;
      border-bottom: 1px solid var(--border);
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.9rem;
    }}
    tr:last-child td {{ border-bottom: none; }}
    tr:hover td {{ background: var(--hover-bg); }}
    td:first-child {{ padding-left: 1.25rem; }}
    td.size, td.date {{
      color: var(--text-muted);
      font-size: 0.8rem;
    }}
    a.entry {{
      color: var(--text);
      text-decoration: none;
      display: inline-flex;
      align-items: center;
      gap: 0.5rem;
    }}
    a.entry:hover {{ color: var(--accent); }}
    a.entry.dir {{ color: var(--dir-color); }}
    a.entry.dir:hover {{ opacity: 0.9; }}
    .icon {{
      width: 1.1em;
      height: 1.1em;
      flex-shrink: 0;
    }}
  </style>
</head>
<body>
  <div class="container-fluid">
    <div class="header-row">
      <div class="header-left">
        <h1>{title}</h1>
        <nav class="breadcrumb" aria-label="Breadcrumb">{breadcrumb_html}</nav>
      </div>
      <button type="button" class="theme-toggle" id="theme-toggle" aria-label="Toggle light/dark mode">
        <svg class="theme-icon theme-icon-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/></svg>
        <svg class="theme-icon theme-icon-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
      </button>
    </div>
    <table>
      <thead>
        <tr>
          <th>Name</th>
          <th class="size">Size</th>
          <th class="date">Modified</th>
        </tr>
      </thead>
      <tbody>
        {rows}
      </tbody>
    </table>
  </div>
  <script>
    (function() {{
      var key = 'webserve-theme';
      var dark = false;
      function apply() {{
        document.documentElement.setAttribute('data-theme', dark ? 'dark' : 'light');
      }}
      try {{
        var s = localStorage.getItem(key);
        if (s === 'dark') {{ dark = true; }}
      }} catch (e) {{}}
      apply();
      document.getElementById('theme-toggle').addEventListener('click', function() {{
        dark = !dark;
        try {{ localStorage.setItem(key, dark ? 'dark' : 'light'); }} catch (e) {{}}
        apply();
      }});
    }})();
  </script>
</body>
</html>"#,
        title = html_escape(&title),
        breadcrumb_html = breadcrumb,
        rows = rows,
    )
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn percent_encode_path_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ' ' => out.push_str("%20"),
            '/' => out.push_str("%2F"),
            '%' => out.push_str("%25"),
            '#' => out.push_str("%23"),
            '?' => out.push_str("%3F"),
            '&' => out.push_str("%26"),
            '=' => out.push_str("%3D"),
            '+' => out.push_str("%2B"),
            c if c.is_ascii() && !c.is_ascii_alphanumeric() && "-_.!~*'()".contains(c) => {
                out.push(c)
            }
            c if c.is_ascii_alphanumeric() => out.push(c),
            c => {
                for b in c.to_string().as_bytes() {
                    out.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    out
}

fn format_breadcrumb(url_prefix: &str) -> String {
    let path = url_prefix.trim_end_matches('/');
    if path.is_empty() || path == "/" {
        return r#"<a href="/">/</a>"#.to_string();
    }
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut html = String::from(r#"<a href="/">/</a>"#);
    let mut acc = String::from("/");
    for seg in segments.iter() {
        acc.push_str(seg);
        acc.push('/');
        let href = html_escape(&acc);
        let name = html_escape(seg);
        html.push_str(r#"<span aria-hidden="true"> / </span><a href=""#);
        html.push_str(&href);
        html.push_str(r#"">"#);
        html.push_str(&name);
        html.push_str("</a>");
    }
    html
}

fn format_time(t: Option<std::time::SystemTime>) -> String {
    let Some(t) = t else { return "—".to_string() };
    let Ok(d) = t.duration_since(std::time::UNIX_EPOCH) else {
        return "—".to_string();
    };
    let secs = d.as_secs();
    let days = secs / 86400;
    let time = secs % 86400;
    let h = time / 3600;
    let m = (time % 3600) / 60;
    let s = time % 60;
    let (y, month, day) = days_to_ymd(days as u32);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y, month, day, h, m, s
    )
}

fn days_to_ymd(days: u32) -> (u32, u32, u32) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (y, m, d)
}

fn format_size(n: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if n < KB {
        format!("{} B", n)
    } else if n < MB {
        format!("{:.1} KB", n as f64 / KB as f64)
    } else if n < GB {
        format!("{:.1} MB", n as f64 / MB as f64)
    } else {
        format!("{:.1} GB", n as f64 / GB as f64)
    }
}

fn format_entry_row(name: &str, href: &str, is_dir: bool, size: &str, date: &str) -> String {
    let name_esc = html_escape(name);
    let href_esc = html_escape(href);
    let icon = if is_dir {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>"#
    } else {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>"#
    };
    let class = if is_dir { "entry dir" } else { "entry" };
    format!(
        r#"<tr><td><a class="{}" href="{}">{} {}</a></td><td class="size">{}</td><td class="date">{}</td></tr>"#,
        class,
        href_esc,
        icon,
        name_esc,
        html_escape(size),
        html_escape(date),
    )
}

/// Handles file requests.
///
/// - Serves static files from the given directory.
/// - Provides directory listings if no `index.html` exists.
/// - Falls back to `index.html` if in SPA mode.
/// - Optionally injects a live reload script when `--watch` is enabled.
/// - When `--watch` is on, caches HTML bodies with that script per file path to avoid repeated read+inject work.
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
            r.insert_header((
                actix_web::http::header::LOCATION,
                format!("{}?{}", location, q),
            ));
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
            let url_prefix = if canonical_path == "/" {
                "/".to_string()
            } else {
                format!("{}/", canonical_path)
            };
            let listing = directory_listing(&file_path, &url_prefix).await;
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

    // Inject live reload script into HTML if watch mode is on; use cache to avoid per-request read+inject
    if data.watch {
        if let Some(ext) = named_file.path().extension() {
            if ext == "html" {
                if let Some(ref cache) = data.html_cache {
                    if let Ok(guard) = cache.read() {
                        if let Some(cached) = guard.get(&file_path) {
                            return Ok(HttpResponse::Ok()
                                .content_type("text/html")
                                .body(cached.clone()));
                        }
                    }
                }

                const RELOAD_SCRIPT: &str = r#"<script>
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
</script>"#;

                let read_path = named_file.path().to_path_buf();
                let mut body = match tokio::fs::read(&read_path).await {
                    Ok(b) => b,
                    Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
                };
                body.extend_from_slice(RELOAD_SCRIPT.as_bytes());
                let body_bytes = Bytes::from(body);

                if let Some(ref cache) = data.html_cache {
                    if let Ok(mut guard) = cache.write() {
                        guard.insert(file_path, body_bytes.clone());
                    }
                }
                return Ok(HttpResponse::Ok()
                    .content_type("text/html")
                    .body(body_bytes));
            }
        }
    }

    Ok(named_file.into_response(&req))
}

/// Short poll: 200 + body `reload` if a file changed since last poll; otherwise 204 immediately.
pub async fn reload_poll(data: web::Data<AppState>) -> impl Responder {
    if data.reload_pending.swap(false, Ordering::SeqCst) {
        HttpResponse::Ok().content_type("text/plain").body("reload")
    } else {
        HttpResponse::NoContent().finish()
    }
}
