FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Install dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release --locked

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN useradd -m -u 1000 streamocracy

# Copy binary from builder
COPY --from=builder /app/target/release/streamocracy /usr/local/bin/streamocracy

# Create config directory and set permissions
RUN mkdir -p /app/config && chown -R streamocracy:streamocracy /app

USER streamocracy

# Set default config path environment variable
ENV STREAMOCRACY_CONFIG=/app/config/config.toml

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD pgrep -x streamocracy > /dev/null || exit 1

ENTRYPOINT ["streamocracy"]
