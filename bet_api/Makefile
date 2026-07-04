generate:
	openapi-generator generate -i openapi.yaml -g rust-axum -o generated \
		--additional-properties=packageName=bet_api_generated,packageVersion=0.1.0

build: generate
	cargo build

run: generate
	cargo run