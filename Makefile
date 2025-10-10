PROJECT_ROOT := $(shell pwd)

CARGO_LLVM_COV_FLAGS := \
	--fail-under-functions 100 \
	--fail-under-lines 100 \
	--fail-under-regions 100 \
	--show-missing-lines \
	-q

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
	cargo fmt
	cargo clippy
	cargo test
	mkdir -p target/llvm-cov
	cargo llvm-cov $(CARGO_LLVM_COV_FLAGS)

test-card-store:
	cd $(PROJECT_ROOT)/crates/card-store && \
	cargo fmt && \
	cargo clippy && \
	cargo test && \
	mkdir -p target/llvm-cov && \
	cargo llvm-cov $(CARGO_LLVM_COV_FLAGS)

test-chess-training-pgn-import:
	cd $(PROJECT_ROOT)/crates/chess-training-pgn-import && \
	cargo fmt && \
	cargo clippy && \
	cargo test && \
	mkdir -p target/llvm-cov && \
	cargo llvm-cov $(CARGO_LLVM_COV_FLAGS)

test-scheduler-core:
	cd $(PROJECT_ROOT)/crates/scheduler-core && \
	cargo fmt && \
	cargo clippy && \
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
