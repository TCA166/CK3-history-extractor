
cargo: src/*.rs src/structures/*.rs
	@echo "Building with cargo..."
	cargo build --release

check: src/*.rs src/structures/*.rs
	@echo "Checking with cargo..."
	cargo test --verbose

doc: src/*.rs src/structures/*.rs
	@echo "Building documentation..."
	cargo doc --no-deps --document-private-items

dependencies:
	@echo "Installing dependencies..."
	sudo dnf install rust cargo rustup rust-src
