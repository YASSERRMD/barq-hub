# Build stage
FROM rustlang/rust:nightly AS builder

RUN apt-get update && apt-get install -y musl-tools libssl-dev pkg-config

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source
COPY . .

# Build for release
# ENV OPENSSL_STATIC=1 (Not needed for dynamic linking on debian)
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*


WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/barq-hub ./barq-hub

# Expose port
EXPOSE 4000

# Run
CMD ["./barq-hub"]
