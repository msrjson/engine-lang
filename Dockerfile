# Engine Lang Genesis Reference Compiler & Runtime Container
# Multi-Stage Build: Rust Compiler Builder -> Minimal Runtime with Z3 SMT

# Stage 1: Builder
FROM rust:1-slim-bookworm AS builder

WORKDIR /build

# Install build dependencies and Z3 development headers
RUN apt-get update && apt-get install -y --no-install-recommends \
    z3 \
    libz3-dev \
    pkg-config \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy compiler source code
COPY compiler/Cargo.toml compiler/Cargo.lock ./compiler/
COPY compiler/src ./compiler/src

# Compile release binary
WORKDIR /build/compiler
RUN cargo build --release

# Stage 2: Genesis Runtime Image
FROM debian:bookworm-slim AS runtime

LABEL maintainer="Engine Lang Core Working Group <architects@engine-lang.org>"
LABEL description="Engine Lang Systems Programming Language - Genesis Node"
LABEL version="0.3.1"

# Install runtime dependencies: Z3 SMT solver and CA certificates
RUN apt-get update && apt-get install -y --no-install-recommends \
    z3 \
    libz3-4 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled enginec binary from the builder stage
COPY --from=builder /build/compiler/target/release/enginec /usr/local/bin/enginec
RUN ln -s /usr/local/bin/enginec /usr/local/bin/engc

# Copy project specification, configuration, and canonical examples
COPY Engine.toml ./
COPY docs/ ./docs/
COPY examples/ ./examples/

# Create persistent storage directory for Single-Level Store (.db)
RUN mkdir -p /app/data && chmod 777 /app/data

# Expose ports:
# 8090: HTTP Boundary Web Gateway
# 5432: Embedded PostgreSQL Wire Protocol Bridge (engine-bridge)
# 9443: Distributed Causal Mesh P2P Wire Transport (QUIC / engine-wire)
EXPOSE 8090 5432 9443

# Environment settings
ENV ENGINE_ENV=production
ENV ENGINE_DB_PATH=/app/data/engine.db
ENV RUST_LOG=info

# Default entrypoint: verify causal blog domain, initialize SLS and serve HTTP + Postgres bridge
CMD ["sh", "-c", "enginec init-db --path /app/data/engine.db --size-mib 32 && enginec check /app/examples/04_causal_blog.eng && enginec run /app/examples/04_causal_blog.eng --db /app/data/engine.db && enginec serve --http-port 8090 --pg-port 5432 --host 0.0.0.0 --db /app/data/engine.db"]
