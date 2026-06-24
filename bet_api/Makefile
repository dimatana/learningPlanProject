generate:
	openapi-generator generate -i openapi.yaml -g rust-axum -o generated

build: generate
	cargo build

run: generate
	cargo run