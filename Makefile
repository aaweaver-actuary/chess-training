PROJECT_ROOT := $(shell pwd)


CARGO_CLIPPY_FLAGS := \
	--workspace \
	--all-targets \
	--all-features \
	-- \
	-D clippy::all \
	-D clippy::pedantic

CARGO_LLVM_COV_FLAGS := \
	--fail-under-functions 95 \
	--fail-under-lines 95 \
	--fail-under-regions 95 \
	--show-missing-lines \
	-q

rust-lint:
	cargo fmt 
	cargo clippy $(CARGO_CLIPPY_FLAGS)

.PHONY: test \
test-steps \
test-card-store \
test-chess-training-pgn-import \
test-scheduler-core \
test-web-ui \
test-session-gateway

test: test-steps \
test-card-store \
test-chess-training-pgn-import \
test-scheduler-core \
test-web-ui \
test-session-gateway

test-steps:
	make rust-lint
	cargo test
	mkdir -p target/llvm-cov
	cargo llvm-cov $(CARGO_LLVM_COV_FLAGS)

test-card-store:
	cd $(PROJECT_ROOT)/crates/card-store && \
	cargo fmt --manifest-path $(PROJECT_ROOT)/Cargo.toml --package card-store && \
	cargo clippy --manifest-path $(PROJECT_ROOT)/Cargo.toml -p card-store --all-targets --all-features -- -D clippy::all -D clippy::pedantic && \
	cargo test && \
	mkdir -p target/llvm-cov && \
	cargo llvm-cov $(CARGO_LLVM_COV_FLAGS)

test-chess-training-pgn-import:
	cd $(PROJECT_ROOT)/crates/chess-training-pgn-import && \
	cargo fmt --manifest-path $(PROJECT_ROOT)/Cargo.toml --package chess-training-pgn-import && \
	cargo clippy --manifest-path $(PROJECT_ROOT)/Cargo.toml -p chess-training-pgn-import --all-targets --all-features -- -D clippy::all -D clippy::pedantic && \
	cargo test && \
	mkdir -p target/llvm-cov && \
	cargo llvm-cov $(CARGO_LLVM_COV_FLAGS)

test-scheduler-core:
	cd $(PROJECT_ROOT)/crates/scheduler-core && \
	cargo fmt --manifest-path $(PROJECT_ROOT)/Cargo.toml --package scheduler-core && \
	cargo clippy --manifest-path $(PROJECT_ROOT)/Cargo.toml -p scheduler-core --all-targets --all-features -- -D clippy::all -D clippy::pedantic && \
	cargo test && \
	mkdir -p target/llvm-cov && \
	cargo llvm-cov $(CARGO_LLVM_COV_FLAGS)

test-web-ui:
	cd $(PROJECT_ROOT)/web-ui && \
	npm run format && \
	npm run lint && \
	npm run typecheck && \
	npm run build && \
	npm run test:coverage

test-session-gateway:
	cd $(PROJECT_ROOT)/apps/session-gateway && \
	npm run format && \
	npm run lint && \
	npm run typecheck && \
	npm run build && \
	npm run test:coverage
