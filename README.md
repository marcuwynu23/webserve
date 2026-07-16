<div align="center">

# webserve

**Static file server for local development** — SPA fallback, live reload, configurable host and port.

[Rust](https://www.rust-lang.org/) · [Actix Web](https://actix.rs/) · [Tokio](https://tokio.rs/)

[![GitHub release](https://img.shields.io/github/v/release/marcuwynu23/webserve?logo=github)](https://github.com/marcuwynu23/webserve/releases)
[![License](https://img.shields.io/github/license/marcuwynu23/webserve?logo=github)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/marcuwynu23/webserve)](https://github.com/marcuwynu23/webserve/stargazers)
[![Crates.io](https://img.shields.io/crates/v/webserve)](https://crates.io/crates/webserve)
![Rust](https://img.shields.io/badge/Rust-1.70%2B-dea584?logo=rust)

➡️ **[Read the full user guide →](USER-GUIDE.md)**

</div>

---

## Table of Contents

- [What Is webserve?](#what-is-webserve)
- [Use Cases](#use-cases)
- [Benefits](#benefits-for-developers)
- [Comparison](#advantages-over-other-tools)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [CLI Reference](#cli-commands)
- [Configuration](#configuration)
- [Examples](#examples)
- [CI/CD Integration](#cicd-integration)
- [Development](#development)
- [Architecture](#architecture)
- [User Guide](USER-GUIDE.md)
- [Contributing](CONTRIBUTING.md)

---

## What Is webserve?

**webserve** is a lightweight static file server for local development, built with Rust and Actix Web. It serves files from a directory, generates styled directory listings, supports SPA fallback routing, and offers optional live reload via filesystem watching.

### What It Does

- **Serves** — static files from any directory with automatic MIME type detection
- **Lists** — styled directory listing with breadcrumbs, file sizes, and modified dates
- **Falls back** — SPA mode serves `index.html` for unknown paths (client-side routing)
- **Reloads** — `--watch` injects a live-reload script that polls for file changes
- **Opens** — `--open` launches the default browser after binding
- **Normalizes** — collapses `//`, `.`, and rejects `..` path traversal
- **Binds** — configurable host/port with automatic port retry when in use
- **Themes** — light/dark mode persisted to localStorage

### Why Use It?

| Problem | How webserve Solves It |
|---|---|
| Need a quick static server | **One command** — `webserve` serves the current directory instantly |
| SPA dev server missing | **`--spa`** — unknown routes serve `index.html` for React/Vue/Angular |
| Manual browser refresh | **`--watch`** — automatic reload on file changes |
| Port already in use | **Auto-retry** — increments port until one is free |
| Ugly directory listings | **Styled UI** — full-width, breadcrumbs, dark/light theme |
| Path traversal attacks | **Built-in protection** — `..` segments are rejected |
| Cross-platform needed | **Rust binary** — single executable for Windows, macOS, Linux |

### The Philosophy

1. **Zero configuration, maximum utility.** Defaults work for 90% of use cases. Flags opt into advanced features.
2. **Performance out of the box.** Compiled Rust means sub-millisecond responses and minimal memory usage.
3. **Your workflow stays yours.** No config files, no project lock-in. Point it at any directory and go.

---

## Use Cases

| Scenario | How webserve Helps |
|---|---|
| **Local frontend development** | Serve a `dist/` folder with SPA fallback and live reload |
| **Quick file sharing on LAN** | Bind to `0.0.0.0` and share a link with colleagues or mobile devices |
| **Static site preview** | Serve a built Jekyll/Hugo/11ty site with directory listing |
| **API prototype with SPA** | Serve the frontend while proxying API calls to a separate backend |
| **CI artifact preview** | Serve test coverage reports or build output in a CI pipeline |
| **Teaching web basics** | Point it at an `exercises/` folder with automatic directory listing |

---

## Benefits for Developers

- **Instant startup** — no config files, no npm install, no Docker
- **Single binary** — download and run, no runtime dependencies
- **Cross-platform** — Windows, macOS, Linux from one codebase
- **Safe defaults** — binds to `127.0.0.1` only, blocks path traversal
- **Port auto-retry** — no "port in use" frustration
- **Beautiful directory listings** — light/dark theme, breadcrumbs, size and date columns
- **SPA support** — one flag enables client-side routing fallback
- **Live reload** — filesystem watcher triggers browser refresh, no browser extension needed
- **Open browser** — one flag launches the default browser on start
- **Tiny footprint** — 3–5 MB binary, ~3000 lines of Rust

---

## Advantages Over Other Tools

| Aspect | webserve | `python -m http.server` | `serve` (npm) | `live-server` | manual (nginx) |
|---|---|---|---|---|---|
| **Setup time** | ~5 seconds | ~2 seconds | ~15 seconds | ~15 seconds | ~10 minutes |
| **Single binary** | Yes | No (requires Python) | No (requires Node) | No (requires Node) | No |
| **SPA fallback** | `--spa` flag | Manual workaround | Not built-in | Not built-in | Manual config |
| **Live reload** | `--watch` flag | No | No | Yes | Manual setup |
| **Directory listing** | Styled, themed | Plain text | Basic | None | Manual config |
| **Dark/light theme** | Built-in | No | No | No | Manual CSS |
| **Port auto-retry** | Yes | No | No | No | No |
| **Open browser** | `--open` flag | No | No | Yes | No |
| **Path traversal protection** | Built-in | No | No | No | Manual config |
| **Performance** | Compiled Rust | Interpreted | Node.js | Node.js | C (fastest) |
| **Binary size** | ~3 MB | ~30 MB (Python runtime) | ~40 MB (Node runtime) | ~40 MB (Node runtime) | ~5 MB (nginx) |
| **License** | MIT | PSF | MIT | MIT | BSD |

---

## Installation

**From crates.io (recommended):**

```bash
cargo install webserve
```

**From source:**

```bash
git clone https://github.com/marcuwynu23/webserve.git
cd webserve
cargo install --path .
```

**Pre-built binary:**

Download the latest release for your platform from the [releases page](https://github.com/marcuwynu23/webserve/releases).

**Verify:**

```bash
webserve --help
```

---

## Quick Start

```bash
# Serve the current directory on http://127.0.0.1:8080
webserve

# Serve a production build with SPA and live reload
webserve --dir ./dist --spa --watch

# Share files on your LAN
webserve --host 0.0.0.0 --port 3000 --dir ./public

# Open browser automatically
webserve --open --port 8080
```

Visit `http://127.0.0.1:8080` in your browser.

---

## CLI Commands

### `webserve`

Start the static file server.

```bash
webserve [OPTIONS]
```

| Flag | Short | Default | Description |
|---|---|---|---|
| `--dir` | `-d` | `.` (current dir) | Root directory to serve |
| `--port` | `-p` | `8080` | TCP port (auto-increments if in use) |
| `--host` | `-h` | `127.0.0.1` | Bind address |
| `--spa` | — | off | SPA fallback to `index.html` |
| `--watch` | `-w` | off | Watch files and reload browsers |
| `--open` | — | off | Open default browser to server URL |
| `--no-redirect-dir-slash` | — | off | Disable `/dir` → `/dir/` redirect |

#### Examples

**Serve with custom directory:**

```bash
webserve -d ./my-site
```

**Development workflow (Vite/React/Vue):**

```bash
webserve --dir ./dist --spa --watch --open
```

**LAN sharing:**

```bash
webserve --host 0.0.0.0 --port 3000
```

**Disable trailing slash redirect:**

```bash
webserve --no-redirect-dir-slash
```

---

## Configuration

webserve uses **CLI flags only** (no config file). All options have sensible defaults.

### Precedence

CLI flags → built-in defaults (no config file layer).

### Flag Reference

| Option | Default | Behavior |
|---|---|---|
| `--dir` | Current directory | Must be an existing directory; exits with error if not found |
| `--port` | `8080` | If in use, tries `8081`, `8082`, ... until a free port is found |
| `--host` | `127.0.0.1` | Use `0.0.0.0` to listen on all network interfaces |
| `--spa` | off | Returns `index.html` for any path that doesn't match a file |
| `--watch` | off | Watches the served directory tree; injects reload script into HTML |
| `--open` | off | Opens the default browser; converts `0.0.0.0` to `127.0.0.1` for the URL |
| `--no-redirect-dir-slash` | off | When set, `/dir` serves `index.html` directly instead of redirecting to `/dir/` |

---

## Examples

**Basic usage:**

```bash
webserve
```

Serves the current directory. Directory listing shown if no `index.html`.

**SPA + watch (frontend development):**

```bash
webserve --dir ./dist --spa --watch
```

Typical for React, Vue, Angular, Svelte apps. Any unknown route returns `index.html`. File changes trigger automatic browser reload.

**LAN sharing with custom port:**

```bash
webserve --host 0.0.0.0 --port 3000 --dir ./public
```

Access from other devices on the same network via `http://<your-ip>:3000`.

**Open browser on start:**

```bash
webserve --open
```

Convenient for daily use — saves the "copy URL and paste into browser" step.

---

## CI/CD Integration

### GitHub Actions — Test

```yaml
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --verbose
      - run: cargo test --verbose
```

### GitHub Actions — Release

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

---

## Development

### Prerequisites

| Tool | Version | Purpose |
|---|---|---|
| Rust | stable (1.70+) | Compiler |
| Cargo | stable | Package manager and build tool |

### Commands

```bash
cargo build        # Build debug binary
cargo build --release  # Build release binary
cargo test         # Run all tests
cargo fmt --check  # Check formatting
cargo clippy       # Lint
```

### Git Hooks

```bash
git config core.hooksPath .githooks
```

The pre-push hook is **off by default**. Opt into version checking:

```bash
WEBSERVE_HOOK_VERSION_CHECK=1 git push origin v1.2.1
```

See [`.githooks/README.md`](.githooks/README.md) for details.

### Project Structure

```
webserve/
├── src/
│   ├── main.rs      # Binary entry point, CLI parsing, server setup
│   ├── lib.rs       # Library crate root, module exports
│   ├── serve.rs     # HTTP handlers: file serving, directory listing, reload poll
│   ├── path.rs      # URL normalization, path joining, root validation
│   └── types.rs     # AppState, ServeOptions, DirEntry, StaticDirError
├── tests/
│   ├── test.rs          # Integration tests for serve and reload
│   ├── cli_test.rs      # CLI option parsing tests
│   └── missing_dir.rs   # Error path and security tests
├── .github/workflows/
│   ├── test.yml     # CI: test suite on push/PR
│   ├── release.yml  # CD: build binaries on tag
│   └── publish.yml  # CD: publish to crates.io on tag
├── docs/
│   ├── index.html   # Landing page
│   └── wrangler.jsonc  # Cloudflare Pages config
├── .githooks/       # Optional pre-push hooks
├── Cargo.toml       # Package manifest
└── CHANGELOG.md     # Release history
```

---

## Architecture

webserve follows a simple layered architecture:

1. **CLI layer** (`main.rs`) — parses flags via `structopt`, validates the static root, sets up the filesystem watcher, and starts the Actix Web server with automatic port retry.
2. **HTTP layer** (`serve.rs`) — handles incoming requests: serves static files, generates directory listing HTML, manages the live-reload polling endpoint, and caches injected HTML bodies.
3. **Path layer** (`path.rs`) — normalizes URL paths (collapses `//` and `.`, rejects `..`), joins URL paths to the filesystem root safely, and validates the serve root.
4. **Types layer** (`types.rs`) — defines shared structures: `AppState` (server configuration + runtime state), `ServeOptions` (CLI flag model), `DirEntry` (directory listing item), and `StaticDirError` (validation errors).

Data flow: `HTTP request → path normalization → file lookup → (file found? serve / dir? list / SPA? fallback / 404)`.

---

## License

MIT © [Mark Wayne Menorca](mailto:marcuwynu23@gmail.com)
