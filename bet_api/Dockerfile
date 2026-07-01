FROM rust:latest AS builder
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --package bet_api

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/bet_api /usr/local/bin/bet_api
COPY --from=builder /app/migrations /migrations
COPY --from=builder /app/src/swagger_ui.html /swagger_ui.html
ENV RUST_LOG=info
EXPOSE 3000
CMD ["bet_api"]