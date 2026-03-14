# Git hooks (native `core.hooksPath`)

Hooks live in **`.githooks/`** so they can be committed. Git only runs them if you point **`core.hooksPath`** here.

## One-time setup (per clone)

From the repository root:

```bash
git config core.hooksPath .githooks
```

That setting is **local to this repo** (stored in `.git/config`). It is **not** committed; each clone needs the command once (or add to your global docs / onboarding).

To use hooks in **all** clones by default, you could run the same command after `git clone`, or document it in the main README.

## What runs

| Hook      | When        | What it does |
|-----------|-------------|----------------|
| **pre-push** | Before `git push` | If the push includes a tag `v*` (e.g. `v1.2.3`), checks that **tag** (without `v`) equals **`Cargo.toml`** `version` and **`Cargo.lock`** root package `webserve` version. Then runs **`cargo check`** unless `WEBSERVE_HOOK_SKIP_CARGO=1`. |

Pushes that **only** move branches (no `v*` tags) are **not** blocked by this hook.

## Skip (emergency only)

```bash
git push --no-verify ...
```

## Optional: skip `cargo check` but keep version checks

Version checks always run for `v*` tag pushes. To skip only the compile step:

```bash
WEBSERVE_HOOK_SKIP_CARGO=1 git push origin v1.2.3
```

## Windows

Git for Windows runs hooks with `sh`; no extra install needed. If `pre-push` is not executable on Unix, run:

```bash
chmod +x .githooks/pre-push
```
