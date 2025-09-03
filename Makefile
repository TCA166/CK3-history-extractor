
cargo: src/*.rs
	@echo "Building with cargo and including templates in executable..."
	RUSTFLAGS="" cargo build --release --features internal,permissive,tokens

debug: src/*.rs
	@echo "Building with cargo in debug mode..."
	cargo run --features ""

check: src/*.rs
	@echo "Checking with cargo..."
	cargo test --verbose

doc: src/*.rs
	@echo "Building documentation..."
	cargo doc --document-private-items --no-deps --bin ck3_history_extractor

dependencies:
	@echo "Installing dependencies..."
	sudo dnf install rustup rust-src cmake fontconfig-devel

crosscompile:
	@echo "Setting up cross-compilation..."
	rustup target add x86_64-pc-windows-gnu
	sudo dnf install mingw64-gcc

windows: src/*.rs
	@echo "Building for Windows..."
	RUSTFLAGS="" cargo build --target x86_64-pc-windows-gnu --release --features internal,permissive,tokens

clean:
	@echo "Cleaning up..."
	cargo clean

fmt:
	@echo "Formatting code..."
	cargo fmt
