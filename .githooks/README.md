# Git hooks (`core.hooksPath`)

```bash
git config core.hooksPath .githooks
```

## pre-push

| Push | Behavior |
|------|----------|
| **No `v*` tag** | Exits immediately (no delay). |
| **Tag `v*`** | Compares tag ↔ `Cargo.toml` ↔ `Cargo.lock` (sed/awk only — **seconds, not minutes**). |

**`cargo check` does not run by default** (that was slowing pushes). To compile before push:

```bash
WEBSERVE_HOOK_RUN_CARGO=1 git push origin v1.2.3
```

Skip the hook: `git push --no-verify`

## Windows

Uses `sh` (Git Bash). Unix: `chmod +x .githooks/pre-push` if needed.
