FROM rust:1.93-slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY crates/ crates/

RUN cargo build --release --bin fitness-app && \
    cp target/release/fitness-app /app/fitness-app

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/fitness-app /app/fitness-app
COPY config/ /app/config/

EXPOSE 8080

CMD ["/app/fitness-app"]
