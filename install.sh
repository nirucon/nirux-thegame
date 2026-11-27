#!/usr/bin/env bash
# NIRUX installer: builds this source tree with cargo and installs the game
# for the current user in ~/.local (the program, a desktop entry and an
# icon). No system libraries are needed beyond X11, which every NIRUX
# desktop has; sound goes through pw-cat, pacat or aplay.
#
#   ./install.sh            install cargo if it is missing (asks first), build
#                           and install
#   ./install.sh --yes      install missing packages without asking
#   ./install.sh --no-deps  never install system packages
#
# Build packages per distribution (installed with sudo when missing):
#   Debian/Ubuntu: cargo   Arch: rust   Void: cargo
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"
BIN="$HOME/.local/bin"
DATA="${XDG_DATA_HOME:-$HOME/.local/share}"
APPS="$DATA/applications"
ICONS="$DATA/icons/hicolor/scalable/apps"
B=$'\033[1m'; G=$'\033[32m'; Y=$'\033[33m'; R=$'\033[31m'; N=$'\033[0m'

ASSUME_YES=0
DEPS=1
[[ ${NIRUX_ASSUME_YES:-0} == 1 ]] && ASSUME_YES=1
for arg in "$@"; do
  case "$arg" in
    -y|--yes) ASSUME_YES=1 ;;
    --no-deps) DEPS=0 ;;
    -h|--help) sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) printf "%sERROR%s unknown option: %s (try --help)\n" "$R" "$N" "$arg"; exit 2 ;;
  esac
done
[[ $(id -u) != 0 ]] || { echo "Run ./install.sh as your own user, not root." >&2; exit 1; }

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
printf "%sNIRUX %s%s: the game, built from source\n\n" "$B" "$VERSION" "$N"

# A rustup toolchain in ~/.cargo/bin counts too.
[[ -x "$HOME/.cargo/bin/cargo" ]] && export PATH="$PATH:$HOME/.cargo/bin"

OS_ID=unknown; OS_LIKE=""
if [[ -r /etc/os-release ]]; then
  OS_ID="$(sed -n 's/^ID=//p' /etc/os-release | tr -d '"')"
  OS_LIKE=" $(sed -n 's/^ID_LIKE=//p' /etc/os-release | tr -d '"') "
fi
if [[ "$OS_ID" =~ ^(arch|omarchy|cachyos|endeavouros|manjaro)$ ]] || [[ "$OS_LIKE" == *" arch "* ]]; then PM=pacman
elif [[ "$OS_ID" =~ ^(void|voidlinux)$ ]]; then PM=xbps
elif [[ "$OS_ID" =~ ^(debian|ubuntu|linuxmint|pop)$ ]] || [[ "$OS_LIKE" == *" debian "* ]] || [[ "$OS_LIKE" == *" ubuntu "* ]]; then PM=apt
else PM=unknown
fi
if [[ $PM == unknown ]]; then
  if command -v xbps-install >/dev/null 2>&1; then PM=xbps
  elif command -v pacman >/dev/null 2>&1; then PM=pacman
  elif command -v apt-get >/dev/null 2>&1; then PM=apt
  fi
fi

if ! command -v cargo >/dev/null 2>&1; then
  case $PM in pacman) pkg=rust ;; apt|xbps) pkg=cargo ;; *) pkg="" ;; esac
  printf "DEPENDENCIES (to build)\n  %s•%s %s\n" "$Y" "$N" "${pkg:-cargo}"
  if [[ $DEPS == 0 || -z $pkg ]]; then
    printf "\n%sMissing:%s cargo. Install Rust 1.85 or newer, then run ./install.sh again.\n" "$R" "$N"
    exit 1
  fi
  if [[ $ASSUME_YES == 0 ]]; then
    if [[ ! -t 0 ]]; then
      printf "%sCargo is missing and there is no terminal to ask for permission.%s\nRun in a terminal, or with --yes.\n" "$R" "$N"
      exit 1
    fi
    printf "\nInstall it with sudo? [Y/n] "
    read -r ans
    [[ ${ans:-Y} =~ ^[Yy]$ ]] || { printf "Installation cancelled.\n"; exit 1; }
  fi
  case $PM in
    pacman) sudo pacman -S --needed --noconfirm "$pkg" ;;
    xbps) sudo xbps-install -Sy "$pkg" ;;
    apt) sudo apt-get update; sudo env DEBIAN_FRONTEND=noninteractive apt-get install -y "$pkg" ;;
  esac
  hash -r
fi

# Rust 1.85 or newer (Debian 13 ships 1.85).
rust_ver="$( (rustc --version 2>/dev/null || true) | awk '{print $2}')"
if [[ -z $rust_ver ]] || [[ "$(printf '%s\n%s\n' 1.85.0 "$rust_ver" | sort -V | head -1)" != 1.85.0 ]]; then
  printf "%sERROR%s Rust 1.85 or newer is needed (found %s). Update it (rustup update) or the distribution's cargo.\n" "$R" "$N" "${rust_ver:-none}"
  exit 1
fi

printf "Building NIRUX %s with Rust %s…\n" "$VERSION" "$rust_ver"
cargo build --release --locked
BINARY="${CARGO_TARGET_DIR:-$ROOT/target}/release/nirux-thegame"
"$BINARY" --version | grep -qx "NIRUX $VERSION" || { printf "%sERROR%s the built program does not run.\n" "$R" "$N"; exit 1; }

mkdir -p "$BIN" "$APPS" "$ICONS"
# An atomic rename: a running game keeps its old file.
install -m 0755 "$BINARY" "$BIN/.nirux-thegame.new.$$"
mv -f "$BIN/.nirux-thegame.new.$$" "$BIN/nirux-thegame"
install -m 0644 assets/nirux-thegame.svg "$ICONS/nirux-thegame.svg"
sed "s|^Exec=nirux-thegame|Exec=$BIN/nirux-thegame|" assets/nirux-thegame.desktop > "$APPS/nirux-thegame.desktop"
chmod 0644 "$APPS/nirux-thegame.desktop"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APPS" >/dev/null 2>&1 || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q "$DATA/icons/hicolor" >/dev/null 2>&1 || true
printf "%s✓%s Installed %s\n" "$G" "$N" "$BIN/nirux-thegame"
printf "%s✓%s Desktop entry %s\n" "$G" "$N" "$APPS/nirux-thegame.desktop"

if ! command -v pw-cat >/dev/null 2>&1 && ! command -v pacat >/dev/null 2>&1 && ! command -v aplay >/dev/null 2>&1; then
  printf "%sNote:%s no pw-cat, pacat or aplay found; install PipeWire, pulseaudio-utils or alsa-utils for sound.\n" "$Y" "$N"
fi
printf "\nRun: nirux-thegame   (F11 fullscreen, Esc pause, M mute)\n"
