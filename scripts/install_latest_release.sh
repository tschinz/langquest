#!/usr/bin/env sh
set -eu

REPO="${REPO:-tschinz/langquest}"
APP="${APP:-lq}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Error: required command not found: $1" >&2
    exit 1
  }
}

need_cmd curl
need_cmd tar
need_cmd uname
need_cmd mktemp

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    case "$ARCH" in
      x86_64) TARGET="x86_64-apple-darwin" ;;
      arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
      *)
        echo "Unsupported macOS arch: $ARCH" >&2
        exit 1
        ;;
    esac
    ;;
  Linux)
    case "$ARCH" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
      *)
        echo "Unsupported Linux arch: $ARCH" >&2
        exit 1
        ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS (this script is for macOS/Linux)" >&2
    exit 1
    ;;
esac

API_URL="https://api.github.com/repos/$REPO/releases/latest"
TAG="$(curl -fsSL "$API_URL" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"

if [ -z "$TAG" ]; then
  echo "Could not determine latest release tag from GitHub API." >&2
  exit 1
fi

ASSET="${APP}-${TAG}-${TARGET}.tar.gz"
URL="https://github.com/$REPO/releases/download/$TAG/$ASSET"

TMPDIR="$(mktemp -d)"
ARCHIVE="$TMPDIR/$ASSET"

echo "Installing $APP $TAG for $TARGET"
echo "Downloading: $URL"
curl -fL "$URL" -o "$ARCHIVE"

mkdir -p "$INSTALL_DIR"
tar -xzf "$ARCHIVE" -C "$TMPDIR"

if [ ! -f "$TMPDIR/$APP" ]; then
  echo "Archive did not contain expected binary: $APP" >&2
  exit 1
fi

install -m 0755 "$TMPDIR/$APP" "$INSTALL_DIR/$APP"

echo "Installed to: $INSTALL_DIR/$APP"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo "Note: $INSTALL_DIR is not in PATH."
    echo "Add this line to your shell profile:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    ;;
esac

"$INSTALL_DIR/$APP" --version || true