<div align="center">
  <h1> webserve </h1>
</div>

<p align="center">
  <img src="https://img.shields.io/github/stars/marcuwynu23/webserve.svg" alt="Stars Badge"/>
  <img src="https://img.shields.io/github/forks/marcuwynu23/webserve.svg" alt="Forks Badge"/>
  <img src="https://img.shields.io/github/issues/marcuwynu23/webserve.svg" alt="Issues Badge"/>
  <img src="https://img.shields.io/github/license/marcuwynu23/webserve.svg" alt="License Badge"/>
</p>

A blazing-fast static file and SPA Web server written in **Rust**, powered by [`warp`](https://crates.io/crates/warp). Supports **live reload**, SPA fallback, and directory serving — like `npm serve`, but compiled and production-ready.

---

## 🚀 Features

- ✅ Serve any directory (`-d ./dist`)
- ✅ SPA fallback support (`--spa`)
- ✅ File watching with automatic browser reload (`--watch`)
- ✅ Configurable host and port (`-h`, `-p`)
- ✅ Built with Rust + Tokio for high performance

---

## 📦 Installation

```bash
git clone https://github.com/your-username/webserve
cd webserve
cargo build --release
```

The executable will be located in `target/release/webserve`.

You can also install it globally (requires Rust):

```bash
cargo install --path .
```

---

## 🛠 Usage

```bash
webserve [OPTIONS]
```

### Options

| Flag            | Description                            | Default           |
| --------------- | -------------------------------------- | ----------------- |
| `-d`, `--dir`   | Directory to serve files from          | Current directory |
| `-p`, `--port`  | Port to listen on                      | `8080`            |
| `-h`, `--host`  | Host/IP to bind                        | `127.0.0.1`       |
| `--spa`         | Enable SPA fallback (404 → index.html) | disabled          |
| `-w`, `--watch` | Enable file watching + auto-reload     | disabled          |

---

## 🧪 Example

Serve a Vite/React app from `./dist`, with SPA fallback and live reload:

```bash
webserve -d ./dist --spa --watch
```

---

## 💻 Injected Reload Script

When `--watch` is used, `index.html` is automatically injected with:

```html
<script>
  const socket = new WebSocket(`ws://${location.host}/reload`);
  socket.onmessage = () => location.reload();
</script>
```

---

## 📁 Project Structure

```
webserve/
├── src/
│   └── main.rs         # Web server logic
├── Cargo.toml          # Dependencies
└── README.md
```

---

## 📜 License

MIT © Mark Wayne Menorca
