#!/usr/bin/env bash

set -euo pipefail

REPO_SLUG="${CLAWGUARD_REPO:-legeling/ClawGuard}"
VERSION="${CLAWGUARD_VERSION:-latest}"
INSTALL_DIR="${CLAWGUARD_INSTALL_DIR:-$HOME/.local/bin}"
TMP_DIR="$(mktemp -d)"

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

main() {
  local arch
  local os
  local target
  local resolved_version
  local package_name
  local archive_url

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

  echo "Installing ClawGuard ${resolved_version} for ${target}"
  mkdir -p "${INSTALL_DIR}"

  curl -fsSL "${archive_url}" -o "${TMP_DIR}/${package_name}.tar.gz"
  tar -xzf "${TMP_DIR}/${package_name}.tar.gz" -C "${TMP_DIR}"

  install "${TMP_DIR}/${package_name}/clawguard" "${INSTALL_DIR}/clawguard"

  echo "Installed to ${INSTALL_DIR}/clawguard"
  echo "Run: clawguard --help"
}

main "$@"
