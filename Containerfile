# --- Build stage ---
FROM rust:1.89 as builder
WORKDIR /usr/src/property_management_system 
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*
# Container layer cache step
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin && echo "fn main() {}" > src/bin/management_api_server.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release
# Copy the rest of the project
COPY src ./src
COPY migrations ./migrations
COPY proto ./proto
COPY build.rs ./
# Build release binary
RUN cargo build --release

# --- Runtime stage ---
FROM debian:trixie-slim as runtime
RUN apt-get update \
    && apt-get install -y libssl-dev ca-certificates libpq5 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/local/bin/property_management_system
# Copy the binary from the builder
COPY --from=builder /usr/src/property_management_system/target/release/management_api_server /usr/local/bin/property_management_system/management_api_server
CMD ["/usr/local/bin/property_management_system/management_api_server"]
