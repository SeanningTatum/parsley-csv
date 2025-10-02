#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Building release binaries for table-to-csv...${NC}\n"

# Get version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
echo -e "${GREEN}Version: ${VERSION}${NC}\n"

# Ensure targets are installed
echo "Ensuring Rust targets are installed..."
rustup target add x86_64-apple-darwin aarch64-apple-darwin

# Clean previous dist directory
rm -rf dist
mkdir -p dist

# Build for both macOS architectures
echo -e "\n${BLUE}Building for x86_64-apple-darwin (Intel)...${NC}"
cargo build --release --target x86_64-apple-darwin

echo -e "\n${BLUE}Building for aarch64-apple-darwin (Apple Silicon)...${NC}"
cargo build --release --target aarch64-apple-darwin

# Copy binaries to dist
echo -e "\n${BLUE}Copying binaries to dist/...${NC}"
cp target/x86_64-apple-darwin/release/table-to-csv dist/table-to-csv-macos-x86_64
cp target/aarch64-apple-darwin/release/table-to-csv dist/table-to-csv-macos-aarch64

# Create universal binary
echo -e "\n${BLUE}Creating universal binary...${NC}"
lipo -create \
    target/x86_64-apple-darwin/release/table-to-csv \
    target/aarch64-apple-darwin/release/table-to-csv \
    -output dist/table-to-csv-macos-universal

# Create compressed archives
echo -e "\n${BLUE}Creating compressed archives...${NC}"
tar -czf dist/table-to-csv-v${VERSION}-macos-x86_64.tar.gz -C dist table-to-csv-macos-x86_64
tar -czf dist/table-to-csv-v${VERSION}-macos-aarch64.tar.gz -C dist table-to-csv-macos-aarch64
tar -czf dist/table-to-csv-v${VERSION}-macos-universal.tar.gz -C dist table-to-csv-macos-universal

# Generate checksums
echo -e "\n${BLUE}Generating checksums...${NC}"
cd dist
shasum -a 256 *.tar.gz > SHA256SUMS
cd ..

# Display results
echo -e "\n${GREEN}✅ Build complete!${NC}\n"
echo "Distribution files in dist/:"
ls -lh dist/*.tar.gz
echo ""
cat dist/SHA256SUMS
echo ""
echo -e "${GREEN}Ready to upload!${NC}"

