# webserve User Guide

**webserve** is a lightweight static file server for local development, built with Rust and Actix Web. This guide covers everything from installation to advanced workflows.

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Command Reference](#command-reference)
- [Configuration](#configuration)
- [Concepts](#concepts)
- [CI/CD Integration](#cicd-integration)
- [Workflows](#workflows)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)

---

## Installation

### Prerequisites

| Requirement | Notes |
|---|---|
| Rust (stable) | Install via [rustup](https://rustup.rs/) if building from source |
| Internet connection | For downloading dependencies or pre-built binaries |

### Method 1: Install from crates.io (recommended)

```bash
cargo install webserve
```

### Method 2: Build from source

```bash
git clone https://github.com/marcuwynu23/webserve.git
cd webserve
cargo build --release
```

The binary is at `target/release/webserve` (or `webserve.exe` on Windows).

To install into your Cargo bin path:

```bash
cargo install --path .
```

### Method 3: Download pre-built binary

Download the archive for your platform from the [releases page](https://github.com/marcuwynu23/webserve/releases). Available formats:

| Platform | Archive |
|---|---|
| Windows (x86_64) | `.zip` |
| Linux (x86_64) | `.tar.gz` |
| macOS (x86_64) | `.tar.gz` |

Extract and place the binary in your `PATH`.

### Verify installation

```bash
webserve --help
```

You should see the usage message and option list.

---

## Quick Start

### Serve the current directory

```bash
webserve
```

Opens `http://127.0.0.1:8080` in your browser (copy-paste manually). You'll see a directory listing if no `index.html` exists.

### Serve a specific directory

```bash
webserve -d ./my-project
```

### SPA + live reload (frontend development)

```bash
webserve --dir ./dist --spa --watch --open
```

This is the recommended workflow when developing React, Vue, Angular, or Svelte apps. The server serves the `dist/` folder, falls back to `index.html` for client-side routes, reloads the browser automatically when files change, and opens the browser on start.

### Share files on your LAN

```bash
webserve --host 0.0.0.0 --port 3000
```

Access from other devices at `http://<your-local-ip>:3000`.

---

## Command Reference

### `webserve`

Start the static file server.

```bash
webserve [OPTIONS]
```

| Flag | Short | Default | Description |
|---|---|---|---|
| `--dir` | `-d` | Current directory | Root directory to serve files from |
| `--port` | `-p` | `8080` | TCP port to bind to |
| `--host` | `-h` | `127.0.0.1` | Address to bind to |
| `--spa` | — | `false` | Enable SPA fallback to `index.html` |
| `--watch` | `-w` | `false` | Enable filesystem watching and live reload |
| `--open` | — | `false` | Open default browser after binding |
| `--no-redirect-dir-slash` | — | `false` | Disable trailing slash redirect for directories |

#### `--dir`

Specifies the root directory to serve. Must be an existing directory. Defaults to the current working directory.

```bash
webserve --dir ~/Sites/my-project
webserve -d ./public
```

If the path does not exist, the server exits with: `<path> not found`.

If the path is a file (not a directory), the server exits with: `<path> is not a directory`.

#### `--port`

Sets the TCP port. If the port is already in use, webserve automatically tries the next port (`8081`, `8082`, ...) until a free port is found. Logs each attempt.

```bash
webserve -p 3000
```

#### `--host`

Sets the bind address. Defaults to `127.0.0.1` (local only). Use `0.0.0.0` to listen on all network interfaces.

```bash
webserve -h 0.0.0.0
```

#### `--spa`

Enables Single Page Application mode. When a request does not match any file on disk, the server serves `index.html` from the root directory instead of returning 404. This allows client-side routers to handle the URL.

```bash
webserve --dir ./dist --spa
```

#### `--watch`

Enables filesystem watching and live reload. The server:

1. Watches the served directory tree for changes (creates, modifies, deletes)
2. Injects a JavaScript reload script into all served HTML files
3. The injected script polls `/reload` every 600ms
4. On file change, the poll endpoint returns `reload` and the browser refreshes

```bash
webserve --dir ./dist --watch
```

The reload script is cached per file to avoid re-reading and re-injecting on every request. The cache is cleared when the watcher fires.

#### `--open`

Opens the default browser to the server URL after binding. If `--host` is `0.0.0.0`, the URL uses `127.0.0.1` instead (since `http://0.0.0.0:8080` is not a valid browser URL).

```bash
webserve --open --port 8080
```

#### `--no-redirect-dir-slash`

By default, accessing a directory without a trailing slash (`/docs`) triggers a temporary redirect to `/docs/`. This flag disables that behavior and serves the directory's `index.html` (or listing) directly at the path without a slash.

```bash
webserve --no-redirect-dir-slash
```

### `/reload` endpoint

When `--watch` is enabled, the server exposes a `GET /reload` endpoint used by the injected client-side script.

| Response | Meaning |
|---|---|
| `200 OK` body: `reload` | A file change was detected; the browser should refresh |
| `204 No Content` | No changes since the last poll |

This endpoint is not meant to be called manually; it is consumed by the auto-injected reload script.

---

## Configuration

webserve uses **CLI flags only** — there is no configuration file.

### Precedence

```
CLI flags → built-in defaults
```

### Defaults

| Option | Default | Notes |
|---|---|---|
| `dir` | `.` | Resolved from `std::env::current_dir()` |
| `port` | `8080` | Incremented automatically if in use |
| `host` | `127.0.0.1` | Only local connections by default |
| `spa` | `false` | Must be explicitly enabled |
| `watch` | `false` | Must be explicitly enabled |
| `open` | `false` | Must be explicitly enabled |
| `redirect_dir_slash` | `true` | Can be disabled via `--no-redirect-dir-slash` |

### Runtime state

The server's runtime configuration (`AppState`) is constructed at startup from CLI flags and shared across all request handlers. In watch mode, a `broadcast::channel` and an `AtomicBool` coordinate the filesystem watcher with the `/reload` polling endpoint.

---

## Concepts

### Directory Listing

When a request targets a directory that does not contain an `index.html`, webserve generates an HTML directory listing page. Features:

- **Breadcrumb navigation** — clickable path segments for quick navigation up the tree
- **Folder-first sorting** — directories listed before files, both sorted case-insensitively
- **Metadata columns** — file name (with icon), size (formatted: B, KB, MB, GB), and last modified date
- **Dark/light theme** — toggle button in the header; preference persisted to `localStorage`
- **Responsive layout** — full-width table with minimum-width headers
- **URL percent-encoding** — special characters in file names are properly encoded

### SPA Fallback Mode

In SPA mode (`--spa`), the server follows this resolution order for each request:

1. Check if the normalized path maps to an existing file → serve it
2. If not found, serve `<root>/index.html` (the SPA entry point)
3. If `index.html` does not exist, return 404

This allows frameworks like React Router, Vue Router, or Angular Router to handle URL routing on the client side.

### Live Reload

When `--watch` is enabled, the server:

1. Creates a `notify` filesystem watcher on the served directory (recursive)
2. On any file system event, sets a pending-reload flag and clears the HTML cache
3. Injects a `<script>` tag at the end of every served HTML file
4. The injected script polls `GET /reload` every 600ms
5. When the endpoint returns `reload`, the script calls `location.reload()`

### Path Normalization

All incoming request URLs are normalized before file system access:

- Repeated slashes (`//`) are collapsed to single `/`
- `.` segments are removed (e.g., `/./foo` → `/foo`)
- `..` segments cause a **404 rejection** (security: prevents directory traversal)
- Directory paths without trailing slashes receive a **307 redirect** to the trailing-slash version (unless `--no-redirect-dir-slash` is set)

### Port Auto-Retry

If the requested port is already in use, webserve increments the port number and retries binding. It continues until a free port is found. Logs each attempt:

```
[INFO] Port 8080 in use, trying 8081...
[INFO] Port 8081 in use, trying 8082...
[INFO] Serving on http://127.0.0.1:8082
```

---

## CI/CD Integration

### GitHub Actions — Test Suite

```yaml
name: Test
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]
jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt -- --check
      - run: cargo clippy -- -D warnings
      - run: cargo build --verbose
      - run: cargo test --verbose
```

### GitHub Actions — Release Binaries

```yaml
name: Release
on:
  push:
    tags: "v*"
jobs:
  release:
    runs-on: ${{ matrix.runner }}
    strategy:
      matrix:
        include:
          - os: windows
            runner: windows-latest
            target: x86_64-pc-windows-msvc
            bin: webserve.exe
          - os: linux
            runner: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            bin: webserve
          - os: macos
            runner: macos-latest
            target: x86_64-apple-darwin
            bin: webserve
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: softprops/action-gh-release@v2
        with:
          files: target/${{ matrix.target }}/release/${{ matrix.bin }}
          generate_release_notes: true
```

### GitHub Actions — Publish to crates.io

```yaml
name: Publish
on:
  push:
    tags: "v*"
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: |
          TAG_VERSION="${GITHUB_REF#refs/tags/v}"
          CARGO_VERSION=$(grep '^version' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
          if [ "$TAG_VERSION" != "$CARGO_VERSION" ]; then
            echo "Tag version ($TAG_VERSION) != Cargo.toml version ($CARGO_VERSION)"
            exit 1
          fi
      - run: cargo publish --verbose
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}
```

---

## Workflows

### Monorepo / Multi-site Development

When working on multiple projects simultaneously, run separate instances on different ports:

```bash
# Terminal 1: Project A
webserve --dir ~/project-a/dist --port 3000 --spa --watch

# Terminal 2: Project B
webserve --dir ~/project-b/dist --port 3001 --spa --watch
```

### Frontend + Backend Development

Serve the frontend with webserve while developing an API on a separate port:

```bash
# Terminal 1: Frontend
webserve --dir ./dist --port 5173 --spa --watch

# Terminal 2: API server
cd api && cargo run  # or npm run dev, etc.
```

Configure your frontend's dev proxy to forward API requests to the backend port.

### Static Site Generation Preview

```bash
# Build the site
npx @11ty/eleventy --output=_site

# Preview with directory listing
webserve --dir ./_site --open

# Or with live reload during development
npx @11ty/eleventy --serve
```

### CI Artifact Preview (GitHub Pages)

Generate and serve test coverage reports in CI:

```bash
# Generate coverage
cargo tarpaulin --out Html --output-dir coverage

# Serve with webserve (accessible via GitHub Pages or similar)
webserve --dir ./coverage --port 8000
```

---

## Troubleshooting

| Problem | Cause | Fix |
|---|---|---|
| **"not found" error on start** | `--dir` path does not exist | Create the directory or provide a valid path |
| **"is not a directory" error on start** | `--dir` path points to a file | Use a directory path instead |
| **Port already in use** | Another process is using the port | webserve auto-retries with the next port. Check logs for the actual port used |
| **Browser shows directory listing instead of my app** | No `index.html` in the served directory | Add an `index.html` or use `--spa` to fall back to an existing `index.html` |
| **SPA routes return 404** | `--spa` not enabled | Add `--spa` flag to enable client-side routing fallback |
| **Live reload not working** | `--watch` not enabled | Add `--watch` flag. Verify the HTML pages have the injected `<script>` tag (view page source) |
| **Other devices can't connect** | Server bound to `127.0.0.1` | Use `--host 0.0.0.0` to listen on all interfaces |
| **Slow directory listing on large directories** | No pagination implemented | Consider using nginx for production directory serving with thousands of files |
| **Binary won't run on older OS** | Compiled with newer Rust features | Build with an older Rust target or download a release built for your OS version |
| **"permission denied" on bind** | Port < 1024 on Unix requires root | Use a port ≥ 1024 (e.g., `--port 8080` or `--port 3000`) |
| **Theme toggle not working** | JavaScript disabled or localStorage unavailable | The directory listing UI requires JavaScript for theme toggle. The listing still works without JS |

### Common error messages

| Error | Meaning |
|---|---|
| `<path> not found` | The `--dir` path does not exist on disk |
| `<path> is not a directory` | The `--dir` path exists but is a file, not a directory |
| `<addr> already in use` | The bind address and port are occupied; webserve retries |
| `permission denied binding to <addr>` | Insufficient privileges for the chosen port |
| `address not available: <addr>` | The host address is not valid on this system |
| `no available port` | All ports from the starting port to `65535` are in use |

---

## FAQ

**Q: What is webserve?**
A: A lightweight static file server for local development, written in Rust.

**Q: How is it different from `python -m http.server`?**
A: webserve offers SPA fallback, live reload, styled directory listings with dark/light theme, path traversal protection, port auto-retry, and a single-binary distribution.

**Q: Is webserve production-ready?**
A: webserve is designed for **local development** and **preview** use cases. For production, consider nginx, Caddy, or a CDN.

**Q: Does webserve support HTTPS?**
A: Not built-in. For local HTTPS, use a reverse proxy (Caddy, nginx) or a tool like `mkcert`.

**Q: Can I serve multiple directories at once?**
A: No. Run multiple instances on different ports for different directories.

**Q: How do I enable CORS?**
A: webserve does not set CORS headers. Use a reverse proxy or browser extension during development.

**Q: Why doesn't `--watch` work with my Docker setup?**
A: Filesystem events may not propagate into Docker containers. Use polling-based watchers or disable `--watch` and reload manually.

**Q: How do I uninstall webserve?**
A: Run `cargo uninstall webserve` or delete the binary you downloaded.

**Q: Does webserve use threads or async?**
A: Async I/O via Tokio and Actix Web. The filesystem watcher runs on a separate thread.

**Q: What happens if I use `--dir` with a symlink?**
A: Symlinks are followed by the OS. The path normalization runs against the symlink path, but the filesystem accesses the target.

**Q: Can I use webserve in production CI/CD?**
A: Yes — it's useful for serving test reports, coverage output, or build artifacts during CI pipeline debugging.

**Q: Who maintains webserve?**
A: [Mark Wayne Menorca](mailto:marcuwynu23@gmail.com). Contributions welcome — see [CONTRIBUTING.md](CONTRIBUTING.md).

---

*See the [README](README.md) for a quick overview or [CONTRIBUTING.md](CONTRIBUTING.md) to get involved.*
