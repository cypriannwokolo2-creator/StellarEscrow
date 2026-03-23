# Build stage
FROM rust:1.70-slim as builder

WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy contract
COPY contract ./contract

# Copy indexer
COPY indexer ./indexer

# Build the indexer
RUN cd indexer && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/indexer/target/release/stellar-escrow-indexer /app/indexer

# Copy config
COPY indexer/config.toml /app/config.toml

# Create non-root user
RUN useradd -r -s /bin/false indexer
RUN chown indexer:indexer /app/indexer
USER indexer

EXPOSE 3000

CMD ["./indexer", "--config", "config.toml"]