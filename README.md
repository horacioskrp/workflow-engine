# Workflow engine

A workflow engine written in Rust.

> Status: **Phase 0 scaffold.** The crate graph, workspace wiring, lints and the
> gRPC contract are in place; the crates are documented stubs. No engine logic yet.

## Layout

```
Cargo.toml              # virtual workspace: shared metadata, lints, dep versions
.cargo/config.toml      # `cargo xtask` alias + commented target-cpu for releases
proto/gateway.proto     # the client-facing gRPC contract
crates/
  common/               # ids, time, shared error scaffolding          (foundation)
  proto/                # generated gRPC/protobuf types                (foundation)
  model/                # records, intents, process model              (foundation)
  feel/                 # expression-language parser + evaluator       (foundation)
  journal/              # append-only replicated log                   (storage)
  state/                # keyed state store + snapshots                (storage)
  transport/            # node-to-node networking                      (cluster)
  cluster/              # Raft consensus + membership                  (cluster)
  engine/               # deterministic stream processor               (engine)
  exporter/             # exporter trait + built-ins                   (engine)
  gateway/              # gRPC gateway, routing to partitions          (edge)
  client/               # Rust client SDK                              (edge)
  broker/               # broker daemon binary                         (app)
  cli/                  # admin/client CLI binary                      (app)
  testkit/              # deterministic clock + fakes for tests
xtask/                  # workspace automation (codegen, release)
```

Dependencies flow one way: `common/proto` → `model` → `feel/state/journal/transport`
→ `cluster/engine` → `gateway/client` → `broker`. No cycles.

## Build

```bash
cargo build          # compiles the whole workspace (no C toolchain required)
cargo clippy         # workspace lints (all + pedantic, unsafe forbidden)
cargo test
cargo xtask codegen  # placeholder: will regenerate proto bindings in Phase 0
```

The daemon can opt into the mimalloc allocator (needs a C compiler):

```bash
cargo run -p broker --features mimalloc
```

## Conventions (per the Pragmatic Rust Guidelines)

- One source of truth for versions in `[workspace.dependencies]`; crates use
  `dep.workspace = true` (M-CARGO-WORKSPACE, M-CRATES-IN-WORKSPACE).
- Flat `crates/` folder, one crate per bounded concern (M-CRATES-FLAT-FOLDER);
  crate names are kept unprefixed for brevity.
- Edition 2024, MSRV 1.90 (M-LATEST-EDITION, M-MSRV).
- Library errors are canonical structs with a captured backtrace
  (M-ERRORS-CANONICAL-STRUCTS); the binaries use `anyhow` (M-APP-ERROR).
- Workspace-wide lints, `unsafe` forbidden by default, overrides via `#[expect]`
  (M-STATIC-VERIFICATION, M-LINT-OVERRIDE-EXPECT).
- Test utilities live in their own crate (M-INTEGRATION-TEST-UTILS).

## Next steps (Phase 0)

1. Wire `tonic-build` in `proto/build.rs` to compile `proto/gateway.proto`.
2. Implement `Topology` end-to-end in `gateway` over an in-memory stub.
3. Spike `openraft` (`cluster`) and decide the expression-language strategy (`feel`).
