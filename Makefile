
cargo: src/*.rs src/structures/*.rs
	@echo "Building with cargo and including templates in executable..."
	RUSTFLAGS="--cfg internal" cargo build --release

debug: src/*.rs src/structures/*.rs
	@echo "Building with cargo in debug mode..."
	cargo build

check: src/*.rs src/structures/*.rs
	@echo "Checking with cargo..."
	cargo test --verbose

doc: src/*.rs src/structures/*.rs
	@echo "Building documentation..."
	cargo doc --document-private-items --bin ck3_history_extractor

dependencies:
	@echo "Installing dependencies..."
	sudo dnf install rustup rust-src

crosscompile:
	@echo "Setting up cross-compilation..."
	rustup target add x86_64-pc-windows-gnu
	sudo dnf install mingw64-gcc

windows: src/*.rs src/structures/*.rs
	@echo "Building for Windows..."
	RUSTFLAGS="--cfg internal" cargo build --target x86_64-pc-windows-gnu --release

clean:
	@echo "Cleaning up..."
	cargo clean
