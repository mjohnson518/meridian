# Multi-stage build for minimal production image
# SECURITY: Pin to SHA256 digest before production deployment
# Get digest: docker pull rust:1.75-slim && docker inspect --format='{{index .RepoDigests 0}}' rust:1.75-slim
# Example: FROM rust:1.75-slim@sha256:abc123...
FROM rust:1.75-slim as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --bin meridian-api

# Runtime stage
# SECURITY: Pin to SHA256 digest before production deployment
# Get digest: docker pull debian:bookworm-slim && docker inspect --format='{{index .RepoDigests 0}}' debian:bookworm-slim
FROM debian:bookworm-slim

# Install runtime dependencies (including curl for health check)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 meridian

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/meridian-api /app/meridian-api

# Copy database migrations
COPY crates/db/migrations /app/migrations

# Set ownership
RUN chown -R meridian:meridian /app

USER meridian

# Expose port
EXPOSE 8080

# Health check (matches route in routes.rs)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["/app/meridian-api"]

