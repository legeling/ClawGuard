#!/usr/bin/env bash

set -euo pipefail

VERSION="${VERSION:-0.1.0-dev}"
TARGET_TRIPLE="${TARGET_TRIPLE:-$(rustc -vV | awk '/host:/ {print $2}')}"
OUTPUT_DIR="${OUTPUT_DIR:-artifacts}"
PACKAGE_NAME="clawguard-${VERSION}-${TARGET_TRIPLE}"
PACKAGE_DIR="${OUTPUT_DIR}/${PACKAGE_NAME}"
BINARY_NAME="clawguard"
BUILD_BINARY_PATH="${CLAWGUARD_BUILD_BINARY_PATH:-}"
SKIP_BUILD="${CLAWGUARD_SKIP_BUILD:-0}"
RELEASE_KEY_ID="${CLAWGUARD_RELEASE_KEY_ID:-official-release}"
RELEASE_PRIVATE_KEY_FILE="${CLAWGUARD_RELEASE_PRIVATE_KEY_FILE:-}"
REQUIRE_SIGNATURE="${CLAWGUARD_REQUIRE_SIGNATURE:-0}"

if [[ "${TARGET_TRIPLE}" == *windows* ]]; then
  BINARY_NAME="clawguard.exe"
fi

if [[ "${VERSION}" == v* ]]; then
  VERSION="${VERSION#v}"
  PACKAGE_NAME="clawguard-${VERSION}-${TARGET_TRIPLE}"
  PACKAGE_DIR="${OUTPUT_DIR}/${PACKAGE_NAME}"
fi

sha256_file() {
  local file_path="$1"

  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file_path}" | awk '{print $1}'
  else
    shasum -a 256 "${file_path}" | awk '{print $1}'
  fi
}

build_binary() {
  if [[ -n "${BUILD_BINARY_PATH}" ]]; then
    echo "${BUILD_BINARY_PATH}"
    return
  fi

  echo "target/${TARGET_TRIPLE}/release/${BINARY_NAME}"
}

sign_archive_if_configured() {
  local archive_path="$1"
  local archive_name
  local checksum
  local payload_path
  local signature_bin_path
  local signature_b64
  local manifest_path

  if [[ -z "${RELEASE_PRIVATE_KEY_FILE}" ]]; then
    if [[ "${REQUIRE_SIGNATURE}" == "1" ]]; then
      echo "CLAWGUARD_RELEASE_PRIVATE_KEY_FILE must be set when CLAWGUARD_REQUIRE_SIGNATURE=1" >&2
      exit 1
    fi

    echo "No release signing key configured; skipping signature manifest generation"
    return
  fi

  archive_name="$(basename "${archive_path}")"
  checksum="$(sha256_file "${archive_path}")"
  payload_path="${OUTPUT_DIR}/${PACKAGE_NAME}.sig.payload"
  signature_bin_path="${OUTPUT_DIR}/${PACKAGE_NAME}.sig.bin"
  manifest_path="${OUTPUT_DIR}/${PACKAGE_NAME}.sig"

  cat > "${payload_path}" <<EOF
version=${VERSION}
package_name=${PACKAGE_NAME}
archive_name=${archive_name}
sha256=${checksum}
key_id=${RELEASE_KEY_ID}
EOF

  openssl pkeyutl \
    -sign \
    -rawin \
    -inkey "${RELEASE_PRIVATE_KEY_FILE}" \
    -in "${payload_path}" \
    -out "${signature_bin_path}"
  signature_b64="$(openssl base64 -A -in "${signature_bin_path}")"

  cat > "${manifest_path}" <<EOF
version=${VERSION}
package_name=${PACKAGE_NAME}
archive_name=${archive_name}
sha256=${checksum}
key_id=${RELEASE_KEY_ID}
signature=${signature_b64}
EOF

  rm -f "${payload_path}" "${signature_bin_path}"
  echo "Signature manifest: ${manifest_path}"
}

if [[ "${SKIP_BUILD}" != "1" ]]; then
  echo "Building release binary for ${PACKAGE_NAME}"
  cargo build --release --target "${TARGET_TRIPLE}" -p clawguard
fi

mkdir -p "${PACKAGE_DIR}"
cp "$(build_binary)" "${PACKAGE_DIR}/${BINARY_NAME}"
cp "README.md" "${PACKAGE_DIR}/README.md"
cp "docs/cli-installation.md" "${PACKAGE_DIR}/INSTALL.md"
cp "docs/vulnerability-tracker.md" "${PACKAGE_DIR}/VULNERABILITIES.md"

if [ -d "rules" ]; then
  mkdir -p "${PACKAGE_DIR}/rules"
  cp -R rules/. "${PACKAGE_DIR}/rules/"
fi

tar -czf "${OUTPUT_DIR}/${PACKAGE_NAME}.tar.gz" -C "${OUTPUT_DIR}" "${PACKAGE_NAME}"
sign_archive_if_configured "${OUTPUT_DIR}/${PACKAGE_NAME}.tar.gz"

echo "Package directory: ${PACKAGE_DIR}"
echo "Archive: ${OUTPUT_DIR}/${PACKAGE_NAME}.tar.gz"
