FROM rust:latest AS builder
WORKDIR /app

COPY trading-service ./trading-service
COPY bet_api/contracts ./bet_api/contracts

WORKDIR /app/trading-service
ENV SQLX_OFFLINE=true
RUN cargo build --release --package trading-service

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/trading-service/target/release/trading-service /usr/local/bin/trading-service
COPY --from=builder /app/trading-service/migrations /migrations
COPY --from=builder /app/trading-service/src/swagger_ui.html /swagger_ui.html

EXPOSE 3001
CMD ["trading-service"]