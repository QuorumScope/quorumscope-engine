.PHONY: setup fmt lint test check db-up db-down init serve index-once index-watch

setup:
	cargo build

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Set API_TEST_DATABASE_URL to also run the PostgreSQL contract tests.
test:
	cargo test --workspace --all-features

check: fmt lint test
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

db-up:
	docker compose up -d db

db-down:
	docker compose down

init:
	cargo run -p quorumscope-cli -- init

serve:
	cargo run -p quorumscope-cli -- serve

index-once:
	cargo run -p quorumscope-cli -- index once

index-watch:
	cargo run -p quorumscope-cli -- index watch
