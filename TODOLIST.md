# TODO list — webserve

Backlog and ideas. Check items off as you go.

## Git hooks

- [x] `.githooks/pre-push` — **no-op by default**; `WEBSERVE_HOOK_VERSION_CHECK=1` before tag push if wanted
- [x] Team onboarding: run `git config core.hooksPath .githooks` after clone

## CI / release

- [ ] Confirm `publish.yml` Rust setup action (e.g. `dtolnay/rust-toolchain@stable`) if jobs fail before `cargo publish`
- [ ] Optional: skip `cargo publish` when version already on crates.io (avoid red runs on retagged releases)
- [ ] Optional: separate workflows — publish only on new `Cargo.toml` version, release binaries on tag

## crates.io / packaging

- [ ] Keep `[package]` metadata filled: `repository`, `homepage`, `documentation`
- [x] Commit `Cargo.lock` before every publish; tag must match `version` in `Cargo.toml`
- [ ] After each release, bump patch (or minor) for the next publish attempt

## Performance (when needed)

- [ ] Add `Cache-Control` / `ETag` for static assets
- [ ] Optional: gzip/brotli (middleware or document reverse proxy)
- [x] Optional: reduce per-request work for `--watch` HTML (cache injected body or inject once)

## Features (DX)

- [ ] Optional config file (TOML) with CLI overrides
- [ ] CORS headers flag for local API + SPA
- [x] Open browser on start (`--open`)
- [x] Trailing-slash redirect (`--no-redirect-dir-slash` to disable) + normalized URL paths (`//`, `.`, reject `..`)

## Features (production-ish)

- [ ] Optional HTTPS (dev certs via `rustls`) or README section for Caddy/nginx
- [ ] Range requests for large files (video)

## Code quality

- [x] Split `lib.rs` into modules (`serve`, `cli`, `path`) when it grows
- [ ] More integration tests for CLI and error paths

## Docs

- [x] README: `cargo install webserve` from crates.io
- [x] README: troubleshooting (port in use, missing `--dir`, publish/tag flow)

---

*Last updated: session backlog — edit freely.*
