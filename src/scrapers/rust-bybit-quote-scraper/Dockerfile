# Multi-stage selection based on ENV argument
ARG ENV=dev

# Use the shared dependency cache as base
FROM rust:1.89-bookworm AS base
WORKDIR /app

# Copy scraper-specific source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Development stage - runs cargo run directly with pre-compiled dependencies
FROM base AS dev
WORKDIR /app
CMD ["cargo", "run"]

# Build stage for production - builds only the application binary
FROM base AS builder
WORKDIR /app
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
COPY --from=builder /app/target/release/rust-bybit-quote-scraper /usr/local/bin/scraper

# Make binary executable and owned by scraper user
RUN chmod +x /usr/local/bin/scraper \
    && chown scraper:scraper /usr/local/bin/scraper

USER scraper
CMD ["/usr/local/bin/scraper"]

# Multi-stage selection based on ENV argument
FROM ${ENV} AS final