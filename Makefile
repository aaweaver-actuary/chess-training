test:
	cargo fmt
	cargo clippy
	cargo test --workspace
	cargo llvm-cov -p chess-training-pgn-import --fail-under-functions 100 \
	        --fail-under-lines 100 \
	        --fail-under-regions 100 \
	        --show-missing-lines \
	        -q
	npm --prefix web-ui run format:check
	npm --prefix web-ui run lint
	npm --prefix web-ui run typecheck
	npm --prefix web-ui run test:coverage
	npm --prefix apps/session-gateway run format:check
	npm --prefix apps/session-gateway run lint
	npm --prefix apps/session-gateway run typecheck
	npm --prefix apps/session-gateway run test:coverage
