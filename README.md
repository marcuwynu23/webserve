<div align="center">

# webserve

**Static file server for local development** — SPA fallback, optional live reload, configurable host and port.

[Rust](https://www.rust-lang.org/) · [Actix Web](https://actix.rs/) · [Tokio](https://tokio.rs/)

[![GitHub stars](https://img.shields.io/github/stars/marcuwynu23/webserve?style=flat&logo=github)](https://github.com/marcuwynu23/webserve/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/marcuwynu23/webserve?style=flat&logo=github)](https://github.com/marcuwynu23/webserve/network/members)
[![GitHub issues](https://img.shields.io/github/issues/marcuwynu23/webserve?style=flat&logo=github)](https://github.com/marcuwynu23/webserve/issues)
[![License](https://img.shields.io/github/license/marcuwynu23/webserve?style=flat)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/webserve.svg)](https://crates.io/crates/webserve)

[**Repository**](https://github.com/marcuwynu23/webserve) · [**Issues**](https://github.com/marcuwynu23/webserve/issues) · [**Pull requests**](https://github.com/marcuwynu23/webserve/pulls)

</div>

---

## Features

| Capability | Description |
|------------|-------------|
| Static hosting | Serve any folder; optional directory listing when no `index.html` is present |
| SPA mode | `--spa` — unknown paths serve `index.html` (client-side routing) |
| Live reload | `--watch` — filesystem watcher + injected reload script for HTML |
| Binding | Configurable `--host` and `--port` (defaults: `127.0.0.1`, `8080`) |

---

## Requirements

- **Rust** (stable), e.g. via [rustup](https://rustup.rs/)

---

## Installation

**From a clone**

```bash
git clone https://github.com/marcuwynu23/webserve.git
cd webserve
cargo build --release
```

Binary: `target/release/webserve` (or `webserve.exe` on Windows).

**Install into Cargo bin path**

```bash
cargo install --path .
```

---

## Usage

```bash
webserve [OPTIONS]
```

Run `webserve --help` for the full option list.

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--dir` | `-d` | Root directory to serve | Current working directory |
| `--port` | `-p` | TCP port | `8080` |
| `--host` | `-h` | Bind address | `127.0.0.1` |
| `--spa` | — | SPA fallback to `index.html` | off |
| `--watch` | `-w` | Watch files and reload browsers | off |

### Examples

Serve the current directory:

```bash
webserve
```

Serve a production build with SPA and reload (typical for Vite/React/Vue `dist`):

```bash
webserve --dir ./dist --spa --watch
```

Listen on all interfaces (e.g. phone on same LAN):

```bash
webserve --host 0.0.0.0 --port 3000 --dir ./public
```

---

## Development

```bash
cargo build
cargo test
```

**Git hooks (tag + version check):** use the repo’s hooks directory (native `core.hooksPath`):

```bash
git config core.hooksPath .githooks
```

Pushing a tag like `v1.2.3` then requires `Cargo.toml` / `Cargo.lock` / tag versions to match (see [`.githooks/README.md`](.githooks/README.md)).

---

## License

MIT © [Mark Wayne Menorca](mailto:marcuwynu23@gmail.com)
