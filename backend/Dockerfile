# syntax=docker/dockerfile:1.7

FROM --platform=linux/amd64 ghcr.io/succinctlabs/sp1:v5.2.2 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY backend ./backend
COPY program ./program

RUN cargo build --release -p world-id-root-replicator-backend

FROM --platform=linux/amd64 ubuntu:24.04 AS runtime

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates libsqlite3-0 \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/world-id-root-replicator-backend /usr/local/bin/world-id-root-replicator-backend

RUN mkdir -p /data /app/artifacts/proofs

ENV DATABASE_URL=sqlite:///data/world-id-root-replicator.db
ENV LISTEN_ADDR=0.0.0.0:3001

EXPOSE 3001

CMD ["world-id-root-replicator-backend"]
