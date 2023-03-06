#!/bin/bash

# Increase build number by one
# build.number file must contain only the following: build=(num)
source build.number
echo "Current build: $build"
build=$((build+1))
echo "build=$build" >  "build.number"

# Build executables
echo "Building Linux executable"
cargo build -r --target x86_64-unknown-linux-gnu

echo "Building Windows executable"
cargo build -r --target x86_64-pc-windows-gnu

# Copy and rename files
mkdir bin
cp target/x86_64-unknown-linux-gnu/release/forte bin/forte-b$build-linux-x86_64
cp target/x86_64-pc-windows-gnu/release/forte.exe bin/forte-b$build-windows-x86_64.exe
