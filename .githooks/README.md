# Git hooks (`core.hooksPath`)

```bash
git config core.hooksPath .githooks
```

## Why the hook is **off** by default

`git push` was sitting there a long time before the upload started. That’s usually **SSH connecting to GitHub**, **Credential Manager**, or **waiting on a hook** — not Rust compile (your `cargo check` is ~2s).

This **pre-push does nothing unless you opt in**, so normal **`git push`** returns as fast as your network allows.

## Optional: version check before a tag push

```bash
WEBSERVE_HOOK_VERSION_CHECK=1 git push origin v1.2.1
```

Checks tag ↔ `Cargo.toml` ↔ `Cargo.lock`. Optional compile:

```bash
WEBSERVE_HOOK_VERSION_CHECK=1 WEBSERVE_HOOK_RUN_CARGO=1 git push origin v1.2.1
```

## CI still protects you

The **publish** workflow already fails if the tag doesn’t match `Cargo.toml`, so crates.io won’t get a bad release.

## Skip any hook

```bash
git push --no-verify
```

## If push is still slow

Try **`git push --no-verify`**. If it’s **still** slow, the delay is **not** this repo’s hook — check VPN, SSH key, or Git Credential Manager.
