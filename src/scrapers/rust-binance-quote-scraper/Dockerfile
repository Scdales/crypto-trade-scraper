# Multi-stage selection based on ENV argument
ARG ENV=dev

# Include the shared template
FROM scratch AS template-source
COPY Dockerfile.template /

# Use the template with the specific scraper name
FROM template-source AS template
ARG SCRAPER_NAME=rust-binance-quote-scraper

# Base stage with Rust toolchain and common dependencies
FROM rust:1.89-bookworm AS base
WORKDIR /app

# Install system dependencies that might be needed across scrapers
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set common environment variables
ENV RUST_BACKTRACE=1
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# Copy the specific scraper source
COPY rust-binance-quote-scraper/ ./

# Development stage - runs cargo run directly
FROM base AS dev
CMD ["cargo", "run"]

# Dependency caching stage - builds only dependencies
FROM base AS deps
RUN cargo build --release --locked
RUN rm src/*.rs target/release/deps/rust_binance_quote_scraper*

# Build stage for production - builds the actual binary
FROM deps AS builder
COPY rust-binance-quote-scraper/src ./src
RUN cargo build --release --locked

# Production runtime stage - minimal image with just the binary
FROM debian:bookworm-slim AS prod

# Install only runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

# Create non-root user for security
RUN useradd -r -s /bin/false scraper

# Copy the binary from builder stage
COPY --from=builder /app/target/release/rust-binance-quote-scraper /usr/local/bin/scraper

# Make binary executable and owned by scraper user
RUN chmod +x /usr/local/bin/scraper \
    && chown scraper:scraper /usr/local/bin/scraper

USER scraper
CMD ["/usr/local/bin/scraper"]

FROM ${ENV} AS final
