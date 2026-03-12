#!/usr/bin/env bash

set -euo pipefail

INSTALL_DIR="${CLAWGUARD_INSTALL_DIR:-$HOME/.local/bin}"
TARGET="${INSTALL_DIR}/clawguard"

if [ ! -f "${TARGET}" ]; then
  echo "ClawGuard binary not found at ${TARGET}" >&2
  exit 1
fi

rm -f "${TARGET}"
echo "Removed ${TARGET}"
