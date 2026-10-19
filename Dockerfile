# Continuum Genesis Reference Compiler & Runtime Container
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

LABEL maintainer="Continuum Core Working Group <architects@continuum-lang.org>"
LABEL description="Continuum Systems Programming Language - Genesis Node"
LABEL version="0.2.0-genesis"

# Install runtime dependencies: Z3 SMT solver and CA certificates
RUN apt-get update && apt-get install -y --no-install-recommends \
    z3 \
    libz3-4 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled ctmc binary from the builder stage
COPY --from=builder /build/compiler/target/release/ctmc /usr/local/bin/ctmc

# Copy project specification, configuration, and canonical examples
COPY Continuum.toml ./
COPY docs/ ./docs/
COPY examples/ ./examples/

# Create persistent storage directory for Single-Level Store (.nvram)
RUN mkdir -p /app/data && chmod 777 /app/data

# Expose ports:
# 8090: HTTP Boundary Web Gateway
# 5432: Embedded PostgreSQL Wire Protocol Bridge (ctm-bridge)
# 9443: Distributed Causal Mesh P2P Wire Transport (QUIC / ctm-wire)
EXPOSE 8090 5432 9443

# Environment settings
ENV CONTINUUM_ENV=production
ENV CONTINUUM_NVRAM_PATH=/app/data/continuum.nvram
ENV RUST_LOG=info

# Default entrypoint: verify vehicle fleet domain and initialize SLS
CMD ["sh", "-c", "ctmc init-nvram --path /app/data/continuum.nvram --size-mib 32 && ctmc check /app/examples/02_persistent_vehicle_fleet.ctm && ctmc run /app/examples/01_hello_causal.ctm --nvram /app/data/continuum.nvram && ctmc bridge --port 5432"]

