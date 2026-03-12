#!/usr/bin/env bash

set -euo pipefail

VERSION="${VERSION:-0.1.0-dev}"
TARGET_TRIPLE="${TARGET_TRIPLE:-$(rustc -vV | awk '/host:/ {print $2}')}"
OUTPUT_DIR="${OUTPUT_DIR:-artifacts}"
PACKAGE_NAME="clawguard-${VERSION}-${TARGET_TRIPLE}"
PACKAGE_DIR="${OUTPUT_DIR}/${PACKAGE_NAME}"

echo "Building release binary for ${PACKAGE_NAME}"
cargo build --release -p clawguard

mkdir -p "${PACKAGE_DIR}"
cp "target/release/clawguard" "${PACKAGE_DIR}/clawguard"
cp "README.md" "${PACKAGE_DIR}/README.md"
cp "docs/cli-installation.md" "${PACKAGE_DIR}/INSTALL.md"
cp "docs/vulnerability-tracker.md" "${PACKAGE_DIR}/VULNERABILITIES.md"

if [ -d "rules" ]; then
  mkdir -p "${PACKAGE_DIR}/rules"
  cp -R rules/. "${PACKAGE_DIR}/rules/"
fi

tar -czf "${OUTPUT_DIR}/${PACKAGE_NAME}.tar.gz" -C "${OUTPUT_DIR}" "${PACKAGE_NAME}"

echo "Package directory: ${PACKAGE_DIR}"
echo "Archive: ${OUTPUT_DIR}/${PACKAGE_NAME}.tar.gz"
