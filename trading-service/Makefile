generate-server:
	npx @openapitools/openapi-generator-cli generate -i openapi.yaml -g rust-axum -o generated \
		--additional-properties=packageName=trading_api_generated,packageVersion=0.1.0

generate-client:
	npx @openapitools/openapi-generator-cli generate -i openapi.yaml -g rust -o trading-client --additional-properties=packageName=trading_client,packageVersion=0.1.0

generate: generate-server generate-client

build: generate
	cargo build

run: generate
	cargo run