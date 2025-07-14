# -------- Build Stage --------
FROM rust:1.88-bookworm AS builder

# Install OpenSSL dev libraries and tools
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN USER=root cargo new --bin app
WORKDIR /app

# Copy manifest and fetch dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# Copy real source
COPY src ./src
RUN touch ./src/main.rs

# Build your app
RUN cargo build --release

# ---------- Runtime stage ----------
FROM debian:bookworm-slim AS runtime

# Create non-root user
RUN useradd -m appuser

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libssl3 \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /home/appuser

# Copy compiled binary
COPY --from=builder /app/target/release/clearseerr /usr/local/bin/clearseerr

# Use non-root user
USER appuser

ENTRYPOINT ["/usr/local/bin/clearseerr"]
    