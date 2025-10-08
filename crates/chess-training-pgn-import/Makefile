test:
	cargo fmt
	cargo clippy
	cargo test -vvv
	cargo llvm-cov --fail-under-functions 100 \
		--fail-under-lines 100 \
		--fail-under-regions 100 \
		--show-missing-lines \
		-vvv