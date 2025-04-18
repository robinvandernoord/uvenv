#!/bin/sh

set -e

PACKAGE_NAME="uvenv"
INSTALL_DIR="$HOME/.local/bin"
VENVS_DIR="$HOME/.local/uvenv/venvs"
VENV_DIR="$VENVS_DIR/$PACKAGE_NAME"
TMPDIR="$(mktemp -d)"
cd "$TMPDIR"

detect_platform() {
  OS="$(uname -s)"
  ARCH="$(uname -m)"

  case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64 | arm64) ARCH="aarch64" ;;
    *) echo "❌ Unsupported architecture: $ARCH" && exit 1 ;;
  esac

  case "$OS" in
    Linux)
      if ldd --version 2>&1 | grep -qi musl; then
        LIBC="musl"
      else
        LIBC="gnu"
      fi
      TARGET="${ARCH}-unknown-linux-${LIBC}"
      ;;
    Darwin)
      TARGET="${ARCH}-apple-darwin"
      ;;
    *)
      echo "❌ Unsupported OS: $OS" && exit 1
      ;;
  esac

  echo "🔍 Detected target: $TARGET"
}

download_uv() {
  UV_FILENAME="uv-${TARGET}.tar.gz"
  UV_URL="https://github.com/astral-sh/uv/releases/latest/download/${UV_FILENAME}"
  echo "⬇️  Downloading uv from $UV_URL..."
  curl -sSL "$UV_URL" -o uv.tar.gz
  tar -xzf uv.tar.gz
  UV_BIN="./uv-${TARGET}/uv"
  chmod +x "$UV_BIN"
}

create_venv_and_install() {
  echo "📦 Creating virtual environment at $VENV_DIR..."
  mkdir -p "$VENVS_DIR"
  "$UV_BIN" venv "$VENV_DIR"
  export VIRTUAL_ENV="$VENV_DIR"
  echo "📥 Installing $PACKAGE_NAME with uv..."
  "$UV_BIN" pip install "$PACKAGE_NAME" 2> /dev/null
}

link_executable() {
  echo "🔗 Linking executable to $INSTALL_DIR..."
  mkdir -p "$INSTALL_DIR"
  ln -sf "$VENV_DIR/bin/$PACKAGE_NAME" "$INSTALL_DIR/$PACKAGE_NAME"
}

get_shell() {
  SHELL_NAME=$(ps -p $$ -o comm= | sed 's/^-//')
  echo "$SHELL_NAME"
}

get_shell_rc_file() {
  SHELL_NAME=$(get_shell)
  case "$SHELL_NAME" in
    bash) echo "$HOME/.bashrc" ;;
    zsh) echo "$HOME/.zshrc" ;;
    sh) echo "$HOME/.profile" ;;
    *) echo "$HOME/.profile" ;;
  esac
}

add_path_to_rc() {
  printf "\n# Added by uvenv installer\nexport PATH=\"%s:\$PATH\"\n" "$INSTALL_DIR" >> "$RC_FILE"
}

maybe_update_path() {
  echo ""
  echo "🧪 Checking if $INSTALL_DIR is in your PATH..."

  case ":$PATH:" in
    *":$INSTALL_DIR:"*)
      echo "✅ $INSTALL_DIR is already in PATH."
      return
      ;;
  esac

  RC_FILE=$(get_shell_rc_file)
  echo "❓ $INSTALL_DIR is not in your PATH."
  printf "   Would you like to automatically add it to your shell config (%s)? [y/N] " "$RC_FILE"
  read -r REPLY
  case "$REPLY" in
    [yY][eE][sS]|[yY])
      add_path_to_rc
      echo "✅ Added to $RC_FILE"
      echo "❗ Run: source $RC_FILE"
      ;;
    *)
      echo "⚠️  Please add the following line to your shell config manually:"
      echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
      ;;
  esac
}

main() {
  detect_platform
  download_uv
  create_venv_and_install
  link_executable
  maybe_update_path

  echo ""
  echo "🎉 $PACKAGE_NAME installed successfully!"
  echo "➡️  After reloading your shell, you can run it with: $PACKAGE_NAME"
}

main
