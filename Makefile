# Usage:
# make release
# make release-windows
# make clean

# Define the target for building and stripping the release binary
release:
	cargo build --release
#	cargo strip -t target/release/qr-gen
#	strip target/release/qr-gen

# Define the target for cross-compiling and stripping the Windows executable
release-windows:
	cross build --release --target x86_64-pc-windows-gnu
#	x86_64-w64-mingw32-strip target/x86_64-pc-windows-gnu/release/qr-gen.exe
#	find target/x86_64-pc-windows-gnu/release -type f -name '*.exe' -exec x86_64-w64-mingw32-strip {} \;

# Optional: Define a clean target to remove build artifacts
clean:
	cargo clean
