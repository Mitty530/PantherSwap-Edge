# Multi-stage Docker build for PantherSwap Edge
# Production-optimized with security hardening and minimal attack surface

# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -m -u 1001 appuser

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src
COPY config ./config
COPY migrations ./migrations

# Build the application
RUN cargo build --release --bin pantherswap-edge

# Runtime stage - minimal distroless image
FROM gcr.io/distroless/cc-debian12:nonroot

# Copy CA certificates for HTTPS
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the binary
COPY --from=builder /app/target/release/pantherswap-edge /usr/local/bin/pantherswap-edge

# Copy configuration files
COPY --from=builder /app/config /app/config

# Create necessary directories
USER 65532:65532

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/pantherswap-edge", "--health-check"] || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV PANTHERSWAP_SERVER_HOST=0.0.0.0
ENV PANTHERSWAP_SERVER_PORT=8080

# Run the application
ENTRYPOINT ["/usr/local/bin/pantherswap-edge"]
