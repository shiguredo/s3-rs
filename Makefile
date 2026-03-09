.PHONY: test cover check clippy fmt clean

# 全テストを実行する
test:
	cargo test --workspace

# 全テストカバレッジ付きで実行する
cover:
	cargo llvm-cov --tests --workspace

# cargo check を実行する
check:
	cargo check --workspace

# cargo clippy を実行する
clippy:
	cargo clippy --workspace -- -D warnings

# cargo fmt を実行する
fmt:
	cargo fmt --all

# ビルド成果物を削除する
clean:
	cargo clean
