# syntax=docker/dockerfile:1

# ---- build stage -----------------------------------------------------------
# Compiles inside Linux, so neither the host MSVC linker nor Smart App Control
# are involved. Builds only the `node` binary and its workspace dependencies.
FROM rust:1-slim AS build
WORKDIR /src

# protoc is required by tonic-build to compile proto/gateway.proto.
RUN apt-get update \
 && apt-get install -y --no-install-recommends protobuf-compiler \
 && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release --package node

# ---- runtime stage ---------------------------------------------------------
# Minimal Debian with glibc; runs as an unprivileged user.
FROM debian:stable-slim AS runtime
RUN useradd --create-home --uid 10001 app
USER app
COPY --from=build /src/target/release/node /usr/local/bin/node
ENTRYPOINT ["node"]
