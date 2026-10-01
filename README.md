# Workflow engine

A workflow engine written in Rust, organized **by capability** (not by
infrastructure layer).

> Status: **Phase 0 scaffold.** The crate graph, workspace wiring, lints and the
> gRPC contract are in place; the crates are documented stubs. No engine logic yet.

## Layout

```
Cargo.toml              # virtual workspace: shared metadata, lints, dep versions
.cargo/config.toml      # `cargo xtask` alias + commented target-cpu for releases
proto/gateway.proto     # the client-facing gRPC contract
crates/
  # capabilities
  workflow/             # process model + deterministic execution   (the core)
  scheduling/           # timers, deadlines, retry backoff
  tasks/                # work items + dispatch to workers
  messaging/            # signals/messages + correlation
  # supporting
  persistence/          # durable state + history + snapshots
  coordination/         # clustering + replication
  api/                  # external gRPC interface
  feed/                 # outbound history stream
  expr/                 # expression language
  # foundation / edge
  kernel/               # ids, time, telemetry, error scaffolding
  contracts/            # generated API schema types
  node/                 # engine daemon binary
  ctl/                  # operator CLI binary
  sdk/                  # client SDK
  harness/              # deterministic clock + fakes for tests
xtask/                  # workspace automation (codegen, release)
```

Dependencies flow one way from `kernel`/`contracts` up through `persistence` and the
capabilities to `api`, and everything is assembled by `node`. No cycles. See
[docs/architecture.md](docs/architecture.md) for the full graph and runtime model.

## Build

```bash
cargo build          # compiles the whole workspace (no C toolchain required)
cargo clippy         # workspace lints (all + pedantic, unsafe forbidden)
cargo test
cargo xtask codegen  # placeholder: will regenerate proto bindings in Phase 0
```

The node can opt into the mimalloc allocator (needs a C compiler):

```bash
cargo run -p node --features mimalloc
```

## Run with Docker

```bash
docker build -t workflow-engine:dev .
```
```bash
docker run --rm workflow-engine:dev
```

## Conventions (per the Pragmatic Rust Guidelines)

- One source of truth for versions in `[workspace.dependencies]`; crates use
  `dep.workspace = true` (M-CARGO-WORKSPACE, M-CRATES-IN-WORKSPACE).
- Flat `crates/` folder, one crate per capability or support (M-CRATES-FLAT-FOLDER).
- Edition 2024, MSRV 1.90 (M-LATEST-EDITION, M-MSRV).
- Library errors are canonical structs with a captured backtrace
  (M-ERRORS-CANONICAL-STRUCTS); the binaries use `anyhow` (M-APP-ERROR).
- Workspace-wide lints, `unsafe` forbidden by default, overrides via `#[expect]`
  (M-STATIC-VERIFICATION, M-LINT-OVERRIDE-EXPECT).
- Test utilities live in their own crate (M-INTEGRATION-TEST-UTILS).

## Docs

See [docs/](docs/) — [architecture](docs/architecture.md) (constitution & capabilities)
and [roadmap](docs/roadmap.md) (all implementation phases).
