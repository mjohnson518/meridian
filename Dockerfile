# Multi-stage build for minimal production image
# SECURITY: Images pinned to SHA256 digest to prevent supply chain attacks
# To update: docker pull <image> && docker inspect --format='{{index .RepoDigests 0}}' <image>
FROM rust:1.75-slim@sha256:70c2a016184099262fd7cee46f3d35fec3568c45c62f87e37f7f665f766b1f74 as builder

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
# SECURITY: Pinned to SHA256 digest - update periodically for security patches
FROM debian:bookworm-slim@sha256:d5d3f9c23164ea16f31852f95bd5959aad1c5e854332fe00f7b3a20fcc9f635c

# DEVOPS-CRIT-007: Hardened runtime container
# Install runtime dependencies (using wget instead of curl for smaller attack surface)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libpq5 \
    wget \
    && rm -rf /var/lib/apt/lists/* \
    # Remove setuid/setgid binaries for security
    && find / -perm /6000 -type f -exec chmod a-s {} \; 2>/dev/null || true

# Create app user with no shell access
RUN useradd -m -u 1001 -s /usr/sbin/nologin meridian

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/meridian-api /app/meridian-api

# Copy database migrations
COPY crates/db/migrations /app/migrations

# Create tmp directory for any runtime needs (read-only root compatible)
RUN mkdir -p /app/tmp && chown -R meridian:meridian /app

# Set ownership and minimal permissions
RUN chmod 550 /app/meridian-api && \
    chown -R meridian:meridian /app

USER meridian

# Security labels
LABEL org.opencontainers.image.title="Meridian API" \
      org.opencontainers.image.description="Multi-currency stablecoin platform API" \
      org.opencontainers.image.source="https://github.com/mjohnson518/meridian" \
      security.readonly-rootfs="recommended" \
      security.no-new-privileges="true"

# Expose port
EXPOSE 8080

# Health check using wget (smaller than curl, no shell needed)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Use exec form to avoid shell
CMD ["/app/meridian-api"]

