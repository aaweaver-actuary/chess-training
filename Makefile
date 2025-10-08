test:
	cargo fmt
	cargo clippy
	cargo test --workspace
	cargo llvm-cov -p chess-training-pgn-import --fail-under-functions 100 \
		--fail-under-lines 100 \
		--show-missing-lines
