.PHONY: setup fmt lint test test-db check db-up db-down db-reset api indexer cli live-verify

setup:
	cargo build

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-features

test-db:
	cargo test --workspace --all-features --test integration

check: fmt lint test
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

db-up:
	docker-compose up -d postgres

db-down:
	docker-compose down

db-reset: db-down db-up
	./scripts/db-reset.sh

api:
	cargo run -p quorumscope-api

indexer:
	cargo run -p quorumscope-indexer

cli:
	cargo run -p quorumscope-cli

live-verify:
	./scripts/live-verify.sh

dev: db-up
	cargo run -p quorumscope-cli -- init
	cargo run -p quorumscope-api
