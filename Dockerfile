# Stage 1: Build
FROM rust:1.83-slim AS builder
WORKDIR /app

# Copy workspace manifests first for caching
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

RUN cargo build --release --bin context-api

# Stage 2: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/context-api /usr/local/bin/context-api

ENV PORT=3001
EXPOSE 3001

CMD ["context-api"]
