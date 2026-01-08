# Themis Contract Governance Action
# Multi-stage build for efficient image size

FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create workspace
WORKDIR /build

# Copy the entire Themis project
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the CLI binary in release mode
RUN cargo build --release -p themis-cli && \
    strip target/release/themis

# Runtime stage - minimal image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/themis /usr/local/bin/themis

# Copy entrypoint script
COPY .github/actions/themis-action/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Set up non-root user for security
RUN useradd -m -s /bin/bash themis
USER themis

ENTRYPOINT ["/entrypoint.sh"]
