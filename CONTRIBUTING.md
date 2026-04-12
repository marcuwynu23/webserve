# Contributing to webserve

Thanks for your interest in improving webserve. This document explains how to report issues, propose changes, and what we expect from pull requests.

## Code of conduct

Everyone participating in this project is expected to follow the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). Please read it before contributing.

## Ways to contribute

- **Bug reports** — reproducible steps, version, and environment help a lot. Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md) when opening an issue.
- **Feature ideas** — describe the problem you are solving and how you imagine it working. Use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.md).
- **Pull requests** — fixes, features, tests, and documentation improvements are welcome. Keep changes focused and easy to review.

For general questions, open an issue with enough context so others can benefit from the answer.

## Development setup

- **Rust** (stable), e.g. via [rustup](https://rustup.rs/).
- Clone the repository and work from the project root.

```bash
cargo build
cargo test
```

Optional git hooks are documented in [README.md](README.md#development) and [`.githooks/README.md`](.githooks/README.md).

## Before you open a pull request

CI runs on pushes and pull requests to `main` and `develop`. Locally, run the same checks so your PR is more likely to pass on the first try:

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo build --verbose
cargo test --verbose
```

Fix any formatting (`cargo fmt`), Clippy warnings, and failing tests before requesting review.

## Pull request guidelines

- **One logical change per PR** when possible. Large refactors are easier if split or discussed first.
- **Describe what changed and why** in the PR body. Link related issues with `Fixes #123` or `See #123` when applicable.
- **Tests** — add or update tests for behavior changes when it makes sense.
- **User-facing changes** — update README or help text if flags or behavior change.
- **Changelog** — release notes are generated from git history; use clear commit messages (conventional style is fine: `feat:`, `fix:`, `docs:`, etc.).

Target branch: open PRs against **`main`** unless a maintainer asks you to use **`develop`**.

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project ([MIT](LICENSE)).
