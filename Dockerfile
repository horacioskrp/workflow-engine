# syntax=docker/dockerfile:1

# ---- build stage -----------------------------------------------------------
# Compiles inside Linux, so neither the host MSVC linker nor Smart App Control
# are involved. Builds only the `broker` binary and its workspace dependencies.
FROM rust:1-slim AS build
WORKDIR /src

# Leverage layer caching: copy manifests first, then sources.
COPY . .
RUN cargo build --release --package broker

# ---- runtime stage ---------------------------------------------------------
# Minimal Debian with glibc; runs as an unprivileged user.
FROM debian:stable-slim AS runtime
RUN useradd --create-home --uid 10001 app
USER app
COPY --from=build /src/target/release/broker /usr/local/bin/broker
ENTRYPOINT ["broker"]
