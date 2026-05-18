# Build Docker image
docker build -t duinodcx-builder -f Dockerfile.build .

# To check your router architecture, run 'uname -m' on the router.
# Common results:
# aarch64 -> Use aarch64-unknown-linux-musl
# armv7l  -> Use armv7-unknown-linux-musleabihf

echo "Building for ARM64 (aarch64)..."
docker run --rm -v "$(pwd):/app" duinodcx-builder cargo zigbuild --release --target aarch64-unknown-linux-musl --manifest-path duinodcx-rs/Cargo.toml

echo "Building for ARMv7 (32-bit HF)..."
docker run --rm -v "$(pwd):/app" duinodcx-builder cargo zigbuild --release --target armv7-unknown-linux-musleabihf --manifest-path duinodcx-rs/Cargo.toml

echo "Building for x86_64..."
docker run --rm -v "$(pwd):/app" duinodcx-builder cargo zigbuild --release --target x86_64-unknown-linux-musl --manifest-path duinodcx-rs/Cargo.toml

echo "Build complete. Binaries are in duinodcx-rs/target/"
