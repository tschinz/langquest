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
need_cmd sed
need_cmd grep
need_cmd install

OS="$(uname -s)"
ARCH="$(uname -m)"
LINUX_LIBC="${LQ_LINUX_LIBC:-auto}"

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
      x86_64) TARGET_ARCH="x86_64" ;;
      aarch64|arm64) TARGET_ARCH="aarch64" ;;
      *)
        echo "Unsupported Linux arch: $ARCH" >&2
        exit 1
        ;;
    esac

    if [ "$LINUX_LIBC" = "auto" ]; then
      if [ -f /etc/alpine-release ]; then
        LINUX_LIBC="musl"
      elif command -v ldd >/dev/null 2>&1 && ldd --version 2>&1 | grep -qi musl; then
        LINUX_LIBC="musl"
      else
        LINUX_LIBC="gnu"
      fi
    fi

    case "$LINUX_LIBC" in
      gnu|musl) ;;
      *)
        echo "Unsupported Linux libc setting: $LINUX_LIBC (expected: auto, gnu, or musl)" >&2
        exit 1
        ;;
    esac

    TARGET="${TARGET_ARCH}-unknown-linux-${LINUX_LIBC}"
    ;;
  *)
    echo "Unsupported OS: $OS (this script is for macOS/Linux)" >&2
    exit 1
    ;;
esac

API_URL="https://api.github.com/repos/$REPO/releases/latest"
RELEASE_JSON="$(curl -fsSL "$API_URL")"

ASSET="$(printf "%s" "$RELEASE_JSON" | sed -n "s/.*\"name\"[[:space:]]*:[[:space:]]*\"\(${APP}-.*-${TARGET}\\.tar\\.gz\)\".*/\1/p" | head -n1)"
URL="$(printf "%s" "$RELEASE_JSON" | sed -n 's/.*"browser_download_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | grep -F "${TARGET}.tar.gz" | head -n1)"

if [ -z "$ASSET" ] || [ -z "$URL" ]; then
  echo "Could not find a Unix asset for target $TARGET in the latest release." >&2
  if [ "$OS" = "Linux" ]; then
    echo "Tip: override Linux libc detection with LQ_LINUX_LIBC=gnu or LQ_LINUX_LIBC=musl." >&2
  fi
  exit 1
fi

TMPDIR="$(mktemp -d)"
ARCHIVE="$TMPDIR/$ASSET"

echo "Installing $APP from latest release for $TARGET"
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
  *":$INSTALL_DIR:"*)
    echo "$INSTALL_DIR is already in PATH."
    ;;
  *)
    echo "Note: $INSTALL_DIR is not in PATH for new shells yet."
    PATH_LINE="case \":\$PATH:\" in *\":$INSTALL_DIR:\"*) ;; *) export PATH=\"\$PATH:$INSTALL_DIR\";; esac"

    if [ -n "${PROFILE_FILE:-}" ]; then
      PROFILES="$PROFILE_FILE"
    else
      SHELL_NAME="$(basename "${SHELL:-sh}")"
      case "$SHELL_NAME" in
        zsh) PROFILES="$HOME/.zshrc $HOME/.zprofile $HOME/.profile" ;;
        bash) PROFILES="$HOME/.bashrc $HOME/.bash_profile $HOME/.profile" ;;
        *) PROFILES="$HOME/.profile" ;;
      esac
    fi

    for PROFILE in $PROFILES; do
      if [ -f "$PROFILE" ] && grep -F "$PATH_LINE" "$PROFILE" >/dev/null 2>&1; then
        echo "PATH entry already present in $PROFILE"
      else
        {
          echo ""
          echo "# Added by lq installer"
          echo "$PATH_LINE"
        } >> "$PROFILE"
        echo "Added PATH entry to $PROFILE"
      fi
    done

    echo "Open a new terminal, or run this now in your current shell:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    ;;
esac

"$INSTALL_DIR/$APP" --version || true