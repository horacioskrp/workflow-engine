# Contributing

## Branching model (gitflow)

| Branch | Purpose |
|---|---|
| `main` | Stable, released state. Only receives merges from `release/*` and `hotfix/*`. |
| `develop` | Integration branch. All features land here first. |
| `feature/*` | New work, branched from `develop`. |
| `release/*` | Release stabilization, branched from `develop`, merged to `main` and back to `develop`. |
| `hotfix/*` | Urgent fixes, branched from `main`, merged to `main` and `develop`. |

**Rule:** never commit or push directly to `develop` or `main`. Work on a
`feature/*` branch and open a Pull Request **into `develop`**.

```bash
git switch develop && git pull
git switch -c feature/my-change
# ... commit ...
git push -u origin feature/my-change
# open a PR: feature/my-change -> develop
```

A PR may merge only when CI is green.

## CI

`.github/workflows/ci.yml` runs on every push and PR to `main`/`develop`:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --all-targets`

`protoc` is installed on the runner because `contracts` compiles
`proto/gateway.proto` via tonic-build.

## Local checks

A C toolchain and `protoc` are required. The simplest reproducible way is Docker:

```bash
docker build -t workflow-engine:dev .
```

Or, with a local Rust toolchain + `protoc`:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

## Conventions

See [docs/architecture.md](docs/architecture.md). Code follows the Pragmatic Rust
Guidelines (lints enforced workspace-wide: clippy `all` + `pedantic`, `unsafe`
forbidden). Commit messages are imperative and scoped (e.g.
`Phase 0 (step 5): implement Topology`).
