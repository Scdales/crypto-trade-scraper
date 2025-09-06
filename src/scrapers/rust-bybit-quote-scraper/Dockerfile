# Base stage with common setup
ARG ENV=dev
FROM rust:latest AS base
WORKDIR /app
COPY ./ ./
ENV RUST_BACKTRACE=1

# Development stage
FROM base AS dev
CMD ["cargo", "run"]

# Build stage for production
FROM base AS builder
RUN cargo build --release

# Production runtime stage
FROM debian:bookworm-slim AS prod

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy only the binary from the builder stage
COPY --from=builder /app/target/release/rust-bybit-quote-scraper ./

# Make sure the binary is executable
RUN chmod +x ./rust-bybit-quote-scraper

CMD ["./rust-bybit-quote-scraper"]

# Default stage selection
FROM ${ENV} AS final
