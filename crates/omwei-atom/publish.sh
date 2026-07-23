#!/bin/bash

# OMWEI 32BSA Trust Hierarchy - Production Release Script
# Industrial-grade release automation for global distribution

set -euo pipefail

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TARGET_VERSION="0.1.2"
CRATE_NAME="omwei-atom"

echo -e "${BLUE}🏭 OMWEI 32BSA Trust Hierarchy - Production Release${NC}"
echo -e "${BLUE}================================================${NC}"
echo -e "${YELLOW}Target Version: ${TARGET_VERSION}${NC}"
echo -e "${YELLOW}Crate Name: ${CRATE_NAME}${NC}"
echo ""

# Safety check: Verify version matches
echo -e "${BLUE}🔍 Safety Check: Version Verification${NC}"
CARGO_VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "'$CRATE_NAME'") | .version')

if [[ "$CARGO_VERSION" != "$TARGET_VERSION" ]]; then
    echo -e "${RED}❌ VERSION MISMATCH: Cargo.toml has version $CARGO_VERSION, expected $TARGET_VERSION${NC}"
    echo -e "${RED}❌ Release aborted. Please update Cargo.toml version.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Version verification passed: $CARGO_VERSION${NC}"
echo ""

# Check Rust toolchain
echo -e "${BLUE}🔧 Environment Check: Rust Toolchain${NC}"
RUST_VERSION=$(rustc --version)
echo -e "${YELLOW}Rust version: $RUST_VERSION${NC}"

# Check if rustc is up to date (simplified check)
if ! command -v rustup &> /dev/null; then
    echo -e "${YELLOW}⚠️  Warning: rustup not found. Consider using rustup for toolchain management.${NC}"
else
    echo -e "${GREEN}✅ Rust toolchain check passed${NC}"
fi
echo ""

# Pre-flight checks
echo -e "${BLUE}🚀 Pre-flight Checks${NC}"

echo -e "${YELLOW}🔧 Running cargo fmt --all --check...${NC}"
if cargo fmt --all --check; then
    echo -e "${GREEN}✅ Code formatting check passed${NC}"
else
    echo -e "${RED}❌ Code formatting check failed${NC}"
    echo -e "${RED}❌ Run 'cargo fmt --all' to fix formatting${NC}"
    exit 1
fi

echo -e "${YELLOW}🔧 Running cargo clippy --all-targets -- -D warnings...${NC}"
if cargo clippy --all-targets -- -D warnings; then
    echo -e "${GREEN}✅ Clippy checks passed${NC}"
else
    echo -e "${RED}❌ Clippy checks failed${NC}"
    echo -e "${RED}❌ Fix clippy warnings before publishing${NC}"
    exit 1
fi

echo -e "${YELLOW}🔧 Running cargo test --doc...${NC}"
if cargo test --doc; then
    echo -e "${GREEN}✅ Documentation tests passed${NC}"
else
    echo -e "${RED}❌ Documentation tests failed${NC}"
    echo -e "${RED}❌ Fix documentation test failures${NC}"
    exit 1
fi

echo ""

# Build verification
echo -e "${BLUE}🏗️  Build Verification${NC}"
echo -e "${YELLOW}🔧 Running cargo build --release...${NC}"
if cargo build --release; then
    echo -e "${GREEN}✅ Release build successful${NC}"
else
    echo -e "${RED}❌ Release build failed${NC}"
    exit 1
fi

echo ""

# Dry run publish
echo -e "${BLUE}🧪 Publish Dry Run${NC}"
echo -e "${YELLOW}🔧 Running cargo publish --dry-run...${NC}"
if cargo publish --dry-run; then
    echo -e "${GREEN}✅ Publish dry run successful${NC}"
else
    echo -e "${RED}❌ Publish dry run failed${NC}"
    echo -e "${RED}❌ Fix publish issues before proceeding${NC}"
    exit 1
fi

echo ""

# Final confirmation
echo -e "${BLUE}🎯 Ready for Production Release${NC}"
echo -e "${GREEN}✅ All checks passed${NC}"
echo -e "${GREEN}✅ Version: $TARGET_VERSION${NC}"
echo -e "${GREEN}✅ Build verified${NC}"
echo -e "${GREEN}✅ Documentation tests passed${NC}"
echo ""

echo -e "${YELLOW}🚀 Publishing ${CRATE_NAME} v${TARGET_VERSION} to crates.io...${NC}"
echo -e "${YELLOW}⚠️  This action is irreversible. Press Ctrl+C to abort.${NC}"
echo ""

# Countdown for safety
for i in {5..1}; do
    echo -e "${YELLOW}Publishing in $i seconds...${NC}"
    sleep 1
done

echo -e "${YELLOW}🔧 Running cargo publish...${NC}"
if cargo publish; then
    echo -e "${GREEN}🎉 SUCCESS: ${CRATE_NAME} v${TARGET_VERSION} published to crates.io${NC}"
    echo -e "${GREEN}🌐 Available at: https://crates.io/crates/${CRATE_NAME}${NC}"
    echo -e "${GREEN}📚 Documentation at: https://docs.rs/${CRATE_NAME}/${TARGET_VERSION}${NC}"
else
    echo -e "${RED}❌ FAILED: Publish command failed${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}🏭 OMWEI 32BSA Trust Hierarchy - Release Complete${NC}"
echo -e "${BLUE}================================================${NC}"
echo -e "${GREEN}🎯 OMWEI execution integrity format successfully deployed${NC}"
echo -e "${GREEN}🔬 Reference implementation now globally available${NC}"
echo -e "${GREEN}⚡ Zero-latency execution integrity determination active${NC}"
