#!/bin/bash

# Increase build number by one
# build.number file must contain only the following: build=(num)
source build.number
build=$((build+1))
echo "Current build: $build"
echo "build=$build" >  "build.number"

export RUSTFLAGS=""

# Build executables
echo "Building Linux executable"
cargo build -r --target x86_64-unknown-linux-gnu

echo "Building Windows executable"
cargo build -r --target x86_64-pc-windows-gnu

# Copy and rename files
mkdir bin
cp target/x86_64-unknown-linux-gnu/release/forte bin/forte-b$build-linux-x86_64
cp target/x86_64-pc-windows-gnu/release/forte.exe bin/forte-b$build-windows-x86_64.exe

# Build legacy executables
export RUSTFLAGS="-C target-feature=-sse4.1,-sse4.2,-sse4a"

echo "Building legacy Linux executable"
cargo build -r --target x86_64-unknown-linux-gnu

echo "Building legacy Windows executable"
cargo build -r --target x86_64-pc-windows-gnu

# Copy and rename files
mkdir bin
cp target/x86_64-unknown-linux-gnu/release/forte bin/forte-b$build-linux-x86_64-legacy
cp target/x86_64-pc-windows-gnu/release/forte.exe bin/forte-b$build-windows-x86_64-legacy.exe
