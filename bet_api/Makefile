generate:
	openapi-generator-cli generate -i openapi.yaml -g rust-axum -o generated

build: generate
	cargo build

run: generate
	cargo run