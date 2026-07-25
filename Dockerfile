# ---- build ----
FROM rust:1-slim-bookworm AS build
WORKDIR /src

COPY Cargo.toml Cargo.lock ./
COPY teltonika-core/ teltonika-core/
COPY teltonika-rest/ teltonika-rest/

RUN cargo build --release --locked --example probe

# ---- runtime ----
FROM debian:bookworm-slim
# rustls-platform-verifier reads the system trust store; without these the
# TLS handshake fails even before accept_invalid_certs is considered.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=build /src/target/release/examples/probe /usr/local/bin/probe
