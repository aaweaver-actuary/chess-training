test-steps:
	cargo fmt
	cargo clippy
	cargo test 
	cargo llvm-cov \
		--fail-under-functions 100 \
	    --fail-under-lines 100 \
	    --fail-under-regions 100 \
	    --show-missing-lines \
	    -q

test-steps-js:
	npm run format
	npm run lint
	npm run typecheck
	npm run test:coverage

PROJECT_ROOT := $(shell pwd)

test:
	make test-steps

	cd $(PROJECT_ROOT)/crates/card-store 
	cargo fmt
	cargo clippy
	cargo test 
	cargo llvm-cov \
		--fail-under-functions 100 \
	    --fail-under-lines 100 \
	    --fail-under-regions 100 \
	    --show-missing-lines \
	    -q

	cd $(PROJECT_ROOT)/crates/chess-training-pgn-import
	cargo fmt
	cargo clippy
	cargo test 
	cargo llvm-cov \
		--fail-under-functions 100 \
	    --fail-under-lines 100 \
	    --fail-under-regions 100 \
	    --show-missing-lines \
	    -q

	cd $(PROJECT_ROOT)/crates/scheduler-core
	cargo fmt
	cargo clippy
	cargo test 
	cargo llvm-cov \
		--fail-under-functions 100 \
	    --fail-under-lines 100 \
	    --fail-under-regions 100 \
	    --show-missing-lines \
	    -q

	cd $(PROJECT_ROOT)/web-ui
	npm run format
	npm run lint
	npm run typecheck
	npm run test:coverage

	cd $(PROJECT_ROOT)/apps/session-gateway
	npm run format
	npm run lint
	npm run typecheck
	npm run test:coverage
