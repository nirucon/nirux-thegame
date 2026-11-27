#!/bin/sh
# X11 end-to-end checks in a private Xvfb, with HOME and every XDG directory
# in a temporary folder and the sound going to a file: the window opens at
# its size, draws a real picture, F11 toggles fullscreen (no window manager
# runs, so the game covers the screen itself), a game starts and makes
# sound, pause and quit to title work, the settings are saved, and QUIT
# ends the program cleanly. Also runs the headless self test.
#
#   tests/xvfb.sh PATH/TO/nirux-thegame [DISPLAY_NUMBER]
set -eu
BIN=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
N=${2:-$((70 + $$ % 20))}
D=:$N
if [ -e "/tmp/.X11-unix/X$N" ] || [ -e "/tmp/.X$N-lock" ]; then
    echo "display $D is in use; pick another number" >&2
    exit 1
fi
SB=$(mktemp -d)
PIDS=""
cleanup() {
    if [ -n "${NIRUX_TEST_LOG:-}" ] && [ -f "$SB/err" ]; then cp "$SB/err" "$NIRUX_TEST_LOG"; fi
    for p in $PIDS; do kill "$p" 2>/dev/null || true; done
    rm -rf "$SB"
}
trap cleanup EXIT INT TERM
fail() { echo "FAIL: $*" >&2; tail -15 "$SB/err" >&2 || true; exit 1; }
ok() { echo "ok: $*"; }

export HOME="$SB/home" XDG_CONFIG_HOME="$SB/config" XDG_DATA_HOME="$SB/data" \
    XDG_CACHE_HOME="$SB/cache" XDG_STATE_HOME="$SB/state" XDG_RUNTIME_DIR="$SB/run"
mkdir -p "$HOME" "$XDG_CONFIG_HOME/nirux-thegame" "$XDG_DATA_HOME" "$XDG_CACHE_HOME" \
    "$XDG_STATE_HOME" "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
unset WAYLAND_DISPLAY || true
printf 'audio = "cat >> %s/pcm.raw"\nfullscreen = false\n' "$SB" > "$XDG_CONFIG_HOME/nirux-thegame/config.toml"

"$BIN" --selftest 2400 > "$SB/selftest" 2>&1 || { cat "$SB/selftest"; fail "self test"; }
ok "self test"

Xvfb "$D" -screen 0 1280x800x24 -nolisten tcp >/dev/null 2>&1 &
PIDS="$!"
i=0
while [ ! -e "/tmp/.X11-unix/X$N" ]; do
    i=$((i + 1)); [ $i -gt 50 ] && fail "Xvfb did not start"; sleep 0.1
done
export DISPLAY="$D" NIRUX_NO_JOYSTICK=1
"$BIN" > "$SB/out" 2> "$SB/err" &
GP=$!
PIDS="$GP $PIDS"

win=""
i=0
while [ -z "$win" ]; do
    i=$((i + 1)); [ $i -gt 50 ] && fail "no window"
    sleep 0.1
    win=$(xdotool search --name '^NIRUX$' 2>/dev/null | head -1 || true)
done
size() { xwininfo -id "$win" | awk '/Width:/ {w=$2} /Height:/ {h=$2} END {print w "x" h}'; }
sleep 0.5
[ "$(size)" = "960x600" ] || fail "window size $(size)"
ok "window 960x600"

# A real picture: many different colours in the window (xwd from
# x11-apps, or maim).
count_colours() {
    python3 - "$1" "$2" <<'PY'
import struct, sys, zlib
kind, path = sys.argv[1], sys.argv[2]
d = open(path, 'rb').read()
if kind == 'xwd':
    hsize = struct.unpack('>I', d[0:4])[0]
    ncolors = struct.unpack('>I', d[76:80])[0]
    px = d[hsize + ncolors * 12:]
    step = 4
else:
    i, idat = 8, b''
    while i < len(d):
        n, t = struct.unpack('>I4s', d[i:i + 8])
        if t == b'IDAT':
            idat += d[i + 8:i + 8 + n]
        i += 12 + n
    px = zlib.decompress(idat)
    step = 3
print(len(set(px[i:i + step] for i in range(0, len(px) - step, step * 7))))
PY
}
colours=""
if command -v python3 >/dev/null && command -v xwd >/dev/null; then
    xwd -id "$win" -silent > "$SB/shot.xwd"
    colours=$(count_colours xwd "$SB/shot.xwd")
elif command -v python3 >/dev/null && command -v maim >/dev/null; then
    maim -i "$win" "$SB/shot.png"
    colours=$(count_colours png "$SB/shot.png")
fi
if [ -n "$colours" ]; then
    [ "$colours" -ge 8 ] || fail "the title screen looks empty ($colours colours)"
    ok "title drawn ($colours colours)"
fi

key() { xdotool windowfocus --sync "$win" 2>/dev/null || true; xdotool key --delay 80 "$@"; sleep 0.3; }
key F11
sleep 0.5
[ "$(size)" = "1280x800" ] || fail "fullscreen size $(size)"
ok "F11 fullscreen"
key F11
sleep 0.5
[ "$(size)" = "960x600" ] || fail "back from fullscreen $(size)"
ok "F11 back to a window"

before=$(stat -c %s "$SB/pcm.raw" 2>/dev/null || echo 0)
key Return
sleep 4
after=$(stat -c %s "$SB/pcm.raw" 2>/dev/null || echo 0)
[ "$after" -gt "$before" ] || fail "no sound written"
ok "game started, sound flowing ($after bytes)"
# Esc skips the prologue to the sector intro, Return starts the sector.
key Escape
sleep 1
key Return
sleep 1
key Escape
key Down Down Return
sleep 0.5
# The controls screen from the title menu, and back.
key Down Down Down Return
sleep 0.5
key Return
sleep 0.5
key Down Down Down Return
i=0
while kill -0 "$GP" 2>/dev/null; do
    i=$((i + 1)); [ $i -gt 50 ] && fail "QUIT did not end the program"; sleep 0.1
done
wait "$GP" || fail "exit status $?"
ok "quit from the menu"
grep -q '^fullscreen = false' "$XDG_CONFIG_HOME/nirux-thegame/config.toml" || fail "settings not saved"
ok "settings saved"
echo "all X11 checks passed"
