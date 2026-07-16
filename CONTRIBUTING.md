# Contributing to webserve

First off, thank you for considering contributing. This project thrives on community involvement.

- [Code of Conduct](#code-of-conduct)
- [Prerequisites](#prerequisites)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Commit Conventions](#commit-conventions)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)
- [Questions](#questions)

---

## Code of Conduct

Please read our [Code of Conduct](CODE_OF_CONDUCT.md) before participating. We expect all contributors to uphold it.

---

## Prerequisites

| Tool | Version | Purpose |
|---|---|---|
| Rust | stable (1.70+) | Compiler and toolchain |
| Cargo | stable | Package manager, build tool, test runner |
| rustfmt | stable (via rustup) | Code formatting |
| clippy | stable (via rustup) | Linting |

Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify the installation:

```bash
rustc --version
cargo --version
```

---

## Project Structure

```
.
├── Cargo.toml            # Package manifest and dependencies
├── Cargo.lock            # Locked dependency versions
├── CHANGELOG.md          # Release history
├── FUNDING.yml           # Sponsor links
├── LICENSE               # MIT license
├── README.md             # Project overview
├── USER-GUIDE.md         # Comprehensive user documentation
├── CONTRIBUTING.md       # This file
├── TODOLIST.md           # Backlog and ideas
├── .github/
│   └── workflows/
│       ├── test.yml      # CI: test suite on push/PR
│       ├── release.yml   # CD: build binaries on tag
│       └── publish.yml   # CD: publish to crates.io on tag
├── .githooks/
│   ├── README.md         # Hook documentation
│   └── pre-push          # Optional version check hook
├── docs/
│   ├── index.html        # Landing page
│   └── wrangler.jsonc    # Cloudflare Pages config
├── src/
│   ├── main.rs           # Binary entry point
│   ├── lib.rs            # Library crate root
│   ├── serve.rs          # HTTP handlers
│   ├── path.rs           # URL/path normalization
│   └── types.rs          # Shared types
└── tests/
    ├── test.rs           # Integration: serve, reload
    ├── cli_test.rs       # Integration: CLI parsing
    └── missing_dir.rs    # Integration: error paths
```

---

## Development Workflow

### Step 1: Fork and clone

```bash
git clone https://github.com/<your-username>/webserve.git
cd webserve
```

### Step 2: Create a branch

```bash
git checkout -b feat/my-feature
```

Branch naming convention: `<type>/<short-description>` (e.g., `feat/cors-support`, `fix/dir-listing-sort`).

### Step 3: Make changes

Write code, add tests, update documentation. See [Coding Standards](#coding-standards) below.

### Step 4: Run checks locally

```bash
cargo fmt -- --check     # Formatting
cargo clippy -- -D warnings  # Linting
cargo build              # Compiles
cargo test               # All tests pass
```

### Step 5: Commit

See [Commit Conventions](#commit-conventions).

### Step 6: Push and open a PR

```bash
git push origin feat/my-feature
```

Then open a pull request against the `main` branch. See [Pull Request Process](#pull-request-process).

---

## Coding Standards

### Naming

- **Types** — `PascalCase` (`ServeOptions`, `AppState`, `DirEntry`)
- **Functions/variables** — `snake_case` (`serve_file`, `reload_poll`, `static_dir`)
- **Constants** — `SCREAMING_SNAKE_CASE` (`RELOAD_SCRIPT`)
- **Modules** — `snake_case` (`serve.rs`, `path.rs`, `types.rs`)

### Imports

Group and order imports:

```rust
// 1. Standard library
use std::path::PathBuf;

// 2. External crates (alphabetical)
use actix_web::{web, HttpResponse};

// 3. Internal crate
use crate::{AppState, DirEntry};
```

### Error handling

- Return `Result<(), String>` from `main.rs` run function; map errors with `map_err`
- Return `Option` from path operations that can fail (path traversal → `None`)
- Use `StaticDirError` enum for root validation failures
- Log startup info with `println!("[INFO] ...")`
- Print fatal errors to stderr

### Formatting

- Run `cargo fmt` before every commit
- Maximum line length: 100 characters (Rust default)
- Use 4-space indentation (Rust default)

### Documentation

- Document all public items with `///` doc comments
- Include a `//!` module-level doc comment at the top of each source file
- Add code examples to doc comments where helpful
- Update the README or USER-GUIDE when adding or changing user-facing features

---

## Testing

### Running tests

```bash
cargo test                  # All tests
cargo test -- --nocapture   # With stdout/stderr output
cargo test cli              # CLI tests only
cargo test missing_dir      # Error path tests only
```

### Test patterns

- **Unit tests** — test individual functions in their module (e.g., `normalize_url_path`, `validate_static_root`)
- **Integration tests** — test the binary via `Command::new(env!("CARGO_BIN_EXE_webserve"))` for CLI behavior, or use `actix_web::test` for HTTP handler tests
- **Error path tests** — test that invalid inputs produce correct errors (missing dir, file as dir, path traversal)

### Test structure

```rust
#[actix_web::test]
async fn test_serve_file_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    // ... setup AppState and test service ...
    let req = test::TestRequest::get().uri("/test.txt").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
}
```

### Adding tests

- Add a new test function for every new feature or bug fix
- Use `tempfile::TempDir` for filesystem-backed tests
- For HTTP handler tests, use `actix_web::test` utilities

---

## Commit Conventions

This project follows [Conventional Commits](https://www.conventionalcommits.org/). The format:

```
<type>(<scope>): <description>
```

### Types

| Type | Usage |
|---|---|
| `feat` | A new feature |
| `fix` | A bug fix |
| `docs` | Documentation only changes |
| `style` | Formatting, missing semicolons, etc. (no code change) |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `test` | Adding or correcting tests |
| `chore` | Build process, CI, or tooling changes |

### Scope

The scope should be the module or area affected. Common scopes:

| Scope | Area |
|---|---|
| `serve` | `src/serve.rs` — HTTP handlers |
| `path` | `src/path.rs` — URL/path normalization |
| `cli` | CLI option parsing (`ServeOptions`) |
| `types` | `src/types.rs` — shared types |
| `ci` | GitHub Actions workflows |
| `hooks` | Git hooks in `.githooks/` |
| `docs` | README, USER-GUIDE, CHANGELOG |
| `server` | `main.rs` — server setup, watcher, binding |

### Examples

```
feat(serve): add CORS header support
fix(path): handle edge case in normalize_url_path with empty segments
docs(README): update installation instructions for Windows
test(cli): add test for --no-redirect-dir-slash flag
refactor(types): rename StaticDirError variants for clarity
chore(ci): pin dtolnay/rust-toolchain to specific version
```

### Breaking changes

Append `BREAKING CHANGE:` footer with a description:

```
feat(serve)!: remove deprecated --no-listing flag

BREAKING CHANGE: The `--no-listing` flag has been removed. Use `--spa` instead.
```

---

## Pull Request Process

### Before submitting

- [ ] Code compiles without warnings (`cargo build`)
- [ ] All tests pass (`cargo test`)
- [ ] Formatting is correct (`cargo fmt -- --check`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] New features include tests
- [ ] Bug fixes include a test that reproduces the issue
- [ ] User-facing changes update relevant docs (README, USER-GUIDE)
- [ ] Commit messages follow [Conventional Commits](#commit-conventions)

### Review criteria

- **Correctness** — does the code do what it says?
- **Safety** — are there any path traversal, DOS, or security issues?
- **Testing** — are edge cases covered?
- **Style** — does it match existing code style?
- **Documentation** — are new flags, features, or behaviors documented?

### What gets merged

- Bug fixes
- New features (with tests and docs)
- Performance improvements
- Documentation improvements
- Test additions/improvements

### What doesn't get merged

- Large refactors without prior discussion (open an issue first)
- Features that significantly increase binary size without proportional benefit
- Changes that break existing behavior without a `BREAKING CHANGE` commit and migration guide

---

## Release Process

1. **Bump version** in `Cargo.toml` and update `CHANGELOG.md`
2. **Commit** — `chore(release): v1.x.x`
3. **Tag** — `git tag v1.x.x`
4. **Push** — `git push origin main --tags`

CI handles the rest:
- `test.yml` runs on every push/PR
- `release.yml` builds binaries for Windows, Linux, macOS and uploads to GitHub Releases
- `publish.yml` publishes to crates.io

The pre-push hook can optionally verify tag/Cargo.toml version consistency:

```bash
WEBSERVE_HOOK_VERSION_CHECK=1 git push origin v1.2.1
```

---

## Questions

| Channel | Link |
|---|---|
| Issues | [github.com/marcuwynu23/webserve/issues](https://github.com/marcuwynu23/webserve/issues) |
| Discussions | [github.com/marcuwynu23/webserve/discussions](https://github.com/marcuwynu23/webserve/discussions) |
| Author | marcuwynu23@gmail.com |

Don't hesitate to open an issue for questions, feature requests, or bug reports before writing code.
