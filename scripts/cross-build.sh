#!/bin/sh
# Cross-compile duinodcx for OpenWRT / iStoreOS with Docker.
set -e
cd "$(dirname "$0")/.."

docker build -t duinodcx-builder -f Dockerfile.build .

# To check your router architecture, run 'uname -m' on the router.
# Common results:
# aarch64 -> aarch64-unknown-linux-musl
# armv7l  -> armv7-unknown-linux-musleabihf
# x86_64  -> x86_64-unknown-linux-musl
for target in aarch64-unknown-linux-musl armv7-unknown-linux-musleabihf x86_64-unknown-linux-musl; do
    echo "Building for $target..."
    # The named volume keeps downloaded crates between runs.
    docker run --rm -v "$(pwd):/app" -v duinodcx-cargo-registry:/usr/local/cargo/registry \
        -e TARGET="$target" duinodcx-builder
done

echo "Build complete. Binaries: target/<target>/release/WinRS-DCX-Management"
