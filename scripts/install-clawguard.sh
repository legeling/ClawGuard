#!/usr/bin/env bash

set -euo pipefail

REPO_SLUG="${CLAWGUARD_REPO:-legeling/ClawGuard}"
VERSION="${CLAWGUARD_VERSION:-latest}"
INSTALL_DIR="${CLAWGUARD_INSTALL_DIR:-$HOME/.local/bin}"
TMP_DIR="$(mktemp -d)"
PUBLIC_KEY_URL="${CLAWGUARD_RELEASE_PUBLIC_KEY_URL:-https://raw.githubusercontent.com/${REPO_SLUG}/main/keys/release-public.pem}"

cleanup() {
  rm -rf "${TMP_DIR}"
}

trap cleanup EXIT

detect_os() {
  local uname_out
  uname_out="$(uname -s)"
  case "${uname_out}" in
    Linux*) echo "unknown-linux-gnu" ;;
    Darwin*) echo "apple-darwin" ;;
    *)
      echo "Unsupported operating system: ${uname_out}" >&2
      exit 1
      ;;
  esac
}

detect_arch() {
  local arch
  arch="$(uname -m)"
  case "${arch}" in
    x86_64|amd64) echo "x86_64" ;;
    arm64|aarch64) echo "aarch64" ;;
    *)
      echo "Unsupported architecture: ${arch}" >&2
      exit 1
      ;;
  esac
}

resolve_version() {
  if [ "${VERSION}" = "latest" ]; then
    curl -fsSL "https://api.github.com/repos/${REPO_SLUG}/releases/latest" \
      | sed -n 's/.*"tag_name": *"v\{0,1\}\([^"]*\)".*/\1/p' \
      | head -n 1
  else
    echo "${VERSION#v}"
  fi
}

sha256_file() {
  local file_path="$1"

  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file_path}" | awk '{print $1}'
  else
    shasum -a 256 "${file_path}" | awk '{print $1}'
  fi
}

manifest_value() {
  local key="$1"
  local manifest_path="$2"

  sed -n "s/^${key}=//p" "${manifest_path}" | head -n 1
}

verify_release() {
  local version="$1"
  local package_name="$2"
  local archive_path="$3"
  local manifest_path="$4"
  local public_key_path="$5"
  local expected_archive_name
  local actual_sha256
  local manifest_sha256
  local payload_path
  local signature_path

  expected_archive_name="${package_name}.tar.gz"

  if [ "$(manifest_value version "${manifest_path}")" != "${version}" ]; then
    echo "Release signature version does not match the requested version." >&2
    exit 1
  fi

  if [ "$(manifest_value package_name "${manifest_path}")" != "${package_name}" ]; then
    echo "Release signature package name does not match the downloaded artifact." >&2
    exit 1
  fi

  if [ "$(manifest_value archive_name "${manifest_path}")" != "${expected_archive_name}" ]; then
    echo "Release signature archive name does not match the downloaded artifact." >&2
    exit 1
  fi

  manifest_sha256="$(manifest_value sha256 "${manifest_path}")"
  actual_sha256="$(sha256_file "${archive_path}")"
  if [ "${manifest_sha256}" != "${actual_sha256}" ]; then
    echo "Downloaded archive checksum does not match the signed manifest." >&2
    exit 1
  fi

  payload_path="${TMP_DIR}/${package_name}.sig.payload"
  signature_path="${TMP_DIR}/${package_name}.sig.bin"

  cat > "${payload_path}" <<EOF
version=$(manifest_value version "${manifest_path}")
package_name=$(manifest_value package_name "${manifest_path}")
archive_name=$(manifest_value archive_name "${manifest_path}")
sha256=$(manifest_value sha256 "${manifest_path}")
key_id=$(manifest_value key_id "${manifest_path}")
EOF

  printf '%s' "$(manifest_value signature "${manifest_path}")" | openssl base64 -d -A -out "${signature_path}"

  openssl pkeyutl \
    -verify \
    -pubin \
    -inkey "${public_key_path}" \
    -rawin \
    -in "${payload_path}" \
    -sigfile "${signature_path}" >/dev/null
}

main() {
  local arch
  local os
  local target
  local resolved_version
  local package_name
  local archive_url
  local signature_url
  local archive_path
  local manifest_path
  local public_key_path

  arch="$(detect_arch)"
  os="$(detect_os)"
  target="${arch}-${os}"
  resolved_version="$(resolve_version)"

  if [ -z "${resolved_version}" ]; then
    echo "Could not resolve a release version from GitHub." >&2
    exit 1
  fi

  package_name="clawguard-${resolved_version}-${target}"
  archive_url="https://github.com/${REPO_SLUG}/releases/download/v${resolved_version}/${package_name}.tar.gz"
  signature_url="https://github.com/${REPO_SLUG}/releases/download/v${resolved_version}/${package_name}.sig"
  archive_path="${TMP_DIR}/${package_name}.tar.gz"
  manifest_path="${TMP_DIR}/${package_name}.sig"
  public_key_path="${TMP_DIR}/release-public.pem"

  echo "Installing ClawGuard ${resolved_version} for ${target}"
  mkdir -p "${INSTALL_DIR}"

  curl -fsSL "${archive_url}" -o "${archive_path}"
  curl -fsSL "${signature_url}" -o "${manifest_path}"
  curl -fsSL "${PUBLIC_KEY_URL}" -o "${public_key_path}"
  verify_release "${resolved_version}" "${package_name}" "${archive_path}" "${manifest_path}" "${public_key_path}"
  tar -xzf "${archive_path}" -C "${TMP_DIR}"

  install "${TMP_DIR}/${package_name}/clawguard" "${INSTALL_DIR}/clawguard"

  echo "Installed to ${INSTALL_DIR}/clawguard"
  echo "Run: clawguard --help"
}

main "$@"
