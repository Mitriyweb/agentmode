#!/bin/bash
# ─────────────────────────────────────────────────────────────────
#  install.sh — One-line installer for agentmode
#
#  Usage: curl -fsSL https://raw.githubusercontent.com/Mitriyweb/agentmode/main/install.sh | bash
# ─────────────────────────────────────────────────────────────────

set -euo pipefail

REPO="Mitriyweb/agentmode"
BIN_DIR="${HOME}/.local/bin"
BIN_NAME="agentmode"

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${BLUE}[install]${NC} $1"; }
ok()  { echo -e "${GREEN}✓${NC} $1"; }
err() { echo -e "${RED}✗${NC} $1" >&2; exit 1; }

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin) PLATFORM="macos" ;;
  *)      err "Unsupported OS: $OS. agentmode only supports macOS." ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH_SUFFIX="x86_64" ;;
  arm64|aarch64) ARCH_SUFFIX="arm64" ;;
  *)             err "Unsupported architecture: $ARCH" ;;
esac

ASSET="agentmode-macos-${ARCH_SUFFIX}.tar.gz"
URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"

log "Detected platform: macOS ${ARCH_SUFFIX}"
log "Downloading ${URL}..."

command -v curl >/dev/null 2>&1 || err "curl is required but not found."
command -v tar >/dev/null 2>&1 || err "tar is required but not found."

mkdir -p "$BIN_DIR"

TMP_DIR="$(mktemp -d)"
TMP_FILE="${TMP_DIR}/${ASSET}"

if curl -fsSL "$URL" -o "$TMP_FILE"; then
  log "Extracting binary..."
  tar -xzf "$TMP_FILE" -C "$TMP_DIR"

  # Find the extracted binary (either inside a folder or directly extracted)
  # The release.yml packages it as: tar czf agentmode-macos-arm64.tar.gz -C arm64 agentmode
  # Which means the binary is directly inside the tar.gz.
  if [ -f "${TMP_DIR}/agentmode" ]; then
    chmod 755 "${TMP_DIR}/agentmode"
    mv "${TMP_DIR}/agentmode" "${BIN_DIR}/${BIN_NAME}"
    ok "Installed ${BIN_NAME} to ${BIN_DIR}/${BIN_NAME}"
  else
    # Fallback to look recursively in case structure changes
    EXTRACTED="$(find "$TMP_DIR" -type f -name "agentmode" | head -n 1)"
    if [ -n "$EXTRACTED" ]; then
      chmod 755 "$EXTRACTED"
      mv "$EXTRACTED" "${BIN_DIR}/${BIN_NAME}"
      ok "Installed ${BIN_NAME} to ${BIN_DIR}/${BIN_NAME}"
    else
      rm -rf "$TMP_DIR"
      err "Binary not found in the downloaded archive."
    fi
  fi
  rm -rf "$TMP_DIR"
else
  rm -rf "$TMP_DIR"
  err "Download failed. Check https://github.com/${REPO}/releases for available binaries."
fi

# Check PATH
if ! echo "$PATH" | tr ':' '\n' | grep -qx "$BIN_DIR"; then
  log ""
  log "Add to your shell profile:"
  log "  export PATH=\"\$PATH:${BIN_DIR}\""
fi

log ""
log "Run: ${BLUE}${BIN_NAME} --help${NC}"
