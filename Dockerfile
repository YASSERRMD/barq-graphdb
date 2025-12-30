# Build stage
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev protobuf-compiler && rm -rf /var/lib/apt/lists/*

# Copy source
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY proto ./proto
COPY src ./src
COPY tests ./tests
COPY benches ./benches

# Build release binaries
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists/*

# Copy binaries from builder
COPY --from=builder /app/target/release/barqg /usr/local/bin/
COPY --from=builder /app/target/release/barqg_server /usr/local/bin/

# Create data directory
RUN mkdir -p /data

# Expose server port
EXPOSE 8080
EXPOSE 50051

# Default command runs the server
CMD ["barqg_server", "--path", "/data", "--host", "0.0.0.0", "--port", "8080"]
