FROM openapitools/openapi-generator-cli:latest AS openapi-generator
WORKDIR /app
COPY . .
RUN openapi-generator-cli generate -i bet_api/openapi.yaml -g rust-axum -o bet_api/generated --additional-properties=packageName=bet_api_generated,packageVersion=0.1.0
RUN openapi-generator-cli generate -i trading-service/openapi.yaml -g rust-axum -o trading-service/generated --additional-properties=packageName=trading_api_generated,packageVersion=0.1.0
RUN openapi-generator-cli generate -i trading-service/openapi.yaml -g rust -o trading-service/trading-client --additional-properties=packageName=trading_client,packageVersion=0.1.0
RUN openapi-generator-cli generate -i bet_api/openapi.yaml -g rust -o bet_api/bet-client --additional-properties=packageName=bet_client,packageVersion=0.1.0

FROM rust:latest AS builder
WORKDIR /app
COPY . .
COPY --from=openapi-generator /app/bet_api/generated /app/bet_api/generated
COPY --from=openapi-generator /app/trading-service/generated /app/trading-service/generated
COPY --from=openapi-generator /app/trading-service/trading-client /app/trading-service/trading-client
COPY --from=openapi-generator /app/bet_api/bet-client /app/bet_api/bet-client
ENV SQLX_OFFLINE=true
RUN cargo build --release --package bet_api

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/bet_api /usr/local/bin/bet_api
COPY --from=builder /app/bet_api/migrations /migrations
COPY --from=builder /app/bet_api/src/swagger_ui.html /swagger_ui.html
ENV RUST_LOG=info
EXPOSE 3000
CMD ["bet_api"]