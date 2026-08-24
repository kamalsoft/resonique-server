.PHONY: test coverage coverage-check

test:
    cargo test --workspace --all-features --all-targets

coverage:
    cargo llvm-cov --workspace --all-features --all-targets --html

coverage-check:
    cargo llvm-cov --workspace --all-features --all-targets --fail-under-lines 100
