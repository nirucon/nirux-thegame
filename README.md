# NIRUX

An original Commodore 64 style shooter for the NIRUX desktop: three kinds
of play in one campaign of eight sectors, chunky pixels in the sixteen C64
colours, raster bars, and a soundtrack played live by a SID-style
synthesiser written for the game.

**Status: complete campaign (0.2.0)** · Part of the [NIRUX](https://github.com/nirucon/nirux) suite · [MIT License](LICENSE)

![NIRUX title screen](docs/screenshots/title.png)

More screenshots: [sector 1, the capital ship](docs/screenshots/sector1.png),
[the Warden](docs/screenshots/boss.png),
[sector 2, isometric flight](docs/screenshots/sector2.png),
[sector 3, on foot](docs/screenshots/sector3.png),
[sector 5, the ice pylons](docs/screenshots/sector5.png),
[sector 6, the burning city](docs/screenshots/sector6.png),
[sector 7, the gauntlet](docs/screenshots/sector7.png),
[the controls screen](docs/screenshots/controls.png).

## About

NIRUX is a game to play during a break. It takes its feel from the shooters
of the 1980s home computers, but everything in it is new: the ships, the
enemies, the levels, the pixel art, the font and the music were all made
for NIRUX, in code and text in this repository. No assets are borrowed.

The campaign has eight sectors of three to six minutes each, about forty
minutes straight through when it goes well. It starts easy and gets harder,
and every sector you reach stays open in **Level select**, so one sector
now and then is a game of its own.

| Sector | Name | Play | Bosses |
| --- | --- | --- | --- |
| 1 | The Outer Rim | space shooter | a gunship, the Warden |
| 2 | Glass Dunes | isometric flight | the Sentinel, the Monolith |
| 3 | Iron Jungle | on foot | a tank, the Gatekeeper |
| 4 | The Shipyards | space shooter | a gunship, the Foundry |
| 5 | Ice Pylons | isometric flight | the Sentinel, the Reactor |
| 6 | Burning City | on foot | a tank, the heli gunship |
| 7 | The Gauntlet | space shooter | the Warden Mk II, a gunship, the Executor |
| 8 | The NIRUX Core | all three, one after the other | the Core Guardian, the Core Spire, the NIRUX Core |

- **Space shooter** (sectors 1, 4, 7): fly right through fighter waves,
  asteroid fields, shipyard gantries and rock tunnels, and over capital
  ships a kilometre long that you take apart turret by turret and
  generator by generator.
- **Isometric flight** (sectors 2, 5): fly up the diagonal over a fortress
  deck. Up and down change altitude; the shadow on the deck and the
  altimeter on the left show where you are. Climb over walls (some of them
  move), fly over or under energy fences (some switch between low and
  high), weave between towers and shoot down bombers.
- **On foot** (sectors 3, 6): fight up the road past riflemen, grenadiers,
  gun nests and rooftop snipers. Fire shoots where you face (hold it to
  keep your aim while you strafe), the special throws a grenade. Locked
  gates open when the attack at them is beaten. In the city, red lights at the rails mean a train.
- **The finale** (sector 8) runs all three in a row, with a stage of the
  final boss at the end of each, and ends with the credits.

## Features

- **Eight sectors** with their own enemies, patterns, mid-bosses and
  bosses, power-ups (weapon levels, shield, bombs, extra lives), score,
  lives, an extra life every 40000 points, game over and a sector clear
  tally, and an ending with credits.
- **Easy, Normal and Hard**: Easy is meant to be finished, Normal to take a
  few attempts, Hard to be a real challenge. A local high score table with
  name entry, and progress saved after every sector.
- **The C64 look**: a 320x200 picture in the sixteen C64 colours (the
  "Pepto" palette), an 8x8 font drawn for the game, parallax starfields,
  raster bars, a chrome logo, a sine scroller, explosions with debris and
  smoke, screen shake and white flashes, and optional CRT scanlines.
- **Any screen**: the picture is scaled by the largest whole number that
  fits (sharp pixels at every size, HiDPI included) and the space around it
  becomes the C64 border, in a colour that changes from sector to sector
  and flashes with the big explosions. Fullscreen on the current monitor by
  default, F11 to switch, and any window size works.
- **Gamepads**: PS3, PS4, PS5, Xbox, 8BitDo and plain USB pads through the
  Linux event devices, plugged in before or during the game. The menus work
  fully with a pad.
- **SID-style sound**: three music voices with triangle, sawtooth, pulse
  and noise, SID timed envelopes, pulse width modulation, sync and ring
  modulation, and a resonant multimode filter with sweeps, plus two voices
  for the sound effects.
- **Light on resources**: one small window, a fixed 60 Hz step that sleeps
  between frames, only the rows of the picture that changed are scaled and
  sent to the X server through shared memory, and sound goes through
  `pw-cat`, `pacat` or `aplay`.
- **Attract mode**: leave the title screen alone and the high scores and a
  self-playing demo of a random sector appear.
- **Menus in the NIRUX theme**: the menu text follows the theme NIRUWM
  uses; the game itself stays in C64 colours.

## Controls

The same list is in the game: **Controls** on the title menu and in the
pause menu. It also shows the name of the gamepad that is connected.

| Action | Keyboard | Gamepad |
| --- | --- | --- |
| Move | arrows or WASD | left stick or d-pad |
| Fire (hold for auto fire) | Space, Ctrl, Z or J | A / cross (the bottom button), right trigger |
| Special: smart bomb (flying) or grenade (on foot) | X, Shift, Alt or K | B / circle (the right button), left trigger |
| Pause | Esc or P | Start |
| Back in menus | Esc | Select / Back |
| Confirm in menus | Enter, Space | A / cross or Start |
| Fullscreen on or off | F11 | |
| Mute | M | |

The other two face buttons and the shoulder buttons work as fire and
special too. In the isometric sectors up and down change altitude, and left and right
move along the deck. On foot, the soldier fires the way he faces; he turns the way he walks,
but while fire is held he keeps his aim, so he can strafe.

Gamepads: the game reads `/dev/input/event*` (and `/dev/input/js*` as a
fallback). It looks for the pad in `/sys/class/input` first, so keyboards,
mice, touchpads and the motion sensors of PlayStation pads are never
opened. Desktops normally let the logged in user read gamepads; if a pad is
found but cannot be read, the game says so on the terminal and on the
Controls screen. The left stick has a dead zone, the d-pad works as a hat
or as buttons, and the pad can be unplugged and plugged in again at any
time.

## The soundtrack

| Song | Where |
| --- | --- |
| Nirux | the title theme, A minor |
| Outer Rim | sector 1, D major, bright and heroic |
| Glass Dunes | sector 2, E minor, mysterious and driving |
| Iron Jungle | sector 3, C minor, a march with snare rolls |
| Shipyards | sector 4, G minor, industrial |
| Ice Pylons | sector 5, F sharp minor, cold and airy |
| Burning City | sector 6, B minor, rock |
| The Gauntlet | sector 7, E minor, fast and frantic |
| The Core | sector 8, D minor, majestic |
| Warden | the bosses of sectors 1 to 7 |
| Heart of Nirux | the final boss, C minor |
| Freedom | the ending and the credits, C major |
| Signal Lost | game over |
| Sector Clear | the fanfare after a sector |

`nirux-thegame --render-wav SONG FILE` writes any of them to a WAV file
(`--list-songs` lists the names).

## Install

```sh
git clone https://github.com/nirucon/nirux-thegame
cd nirux-thegame
./install.sh
```

The installer builds the game with cargo (Rust 1.85 or newer; it offers to
install cargo when it is missing) and puts `nirux-thegame` in
`~/.local/bin`, with a desktop entry and an icon. Run `./uninstall.sh` to
remove it (`--purge` also removes settings, progress and high scores).

Needs: X11 (any window manager, or none), and for sound one of `pw-cat`
(PipeWire), `pacat` (PulseAudio) or `aplay` (ALSA).

## Command line

```text
nirux-thegame                 play (fullscreen unless the settings say otherwise)
nirux-thegame --windowed      play in a window
nirux-thegame --level N       start a game at sector N (1 to 8)
nirux-thegame --demo          start in the attract mode
nirux-thegame --selftest [FRAMES]
                              play every sector on every difficulty with the
                              autopilot, without a window or sound, and check
                              the menus and the music
nirux-thegame --render-wav SONG FILE [SECONDS]
nirux-thegame --list-songs
nirux-thegame --version
```

## Files

- Settings: `$XDG_CONFIG_HOME/nirux-thegame/config.toml` (fullscreen, CRT
  scanlines, music and effects volume, echo, difficulty, and an optional
  `audio` command that plays raw 16-bit stereo PCM from standard input,
  with `%r` for the sample rate).
- Progress and high scores: `$XDG_DATA_HOME/nirux-thegame/save.toml`.

Environment: `NIRUX_NO_JOYSTICK=1` ignores gamepads. `NIRUX_RENDER=1`
lets the X server scale the picture with RENDER instead; it is off by
default because it crashed the X session of an AMD laptop (Xorg with
glamor).

## Development

```sh
cargo fmt --all -- --check
cargo clippy --release --all-targets -- -D warnings
cargo test --release
sh tests/xvfb.sh target/release/nirux-thegame 99
```

For screenshots and testing, `--show controls`, `--show select` and
`--show ending` open a screen directly, `--bot` lets the autopilot play,
and `NIRUX_STAGE=2` or `3` starts sector 8 at a later stage.

`tests/xvfb.sh` runs the game in a private Xvfb with a temporary home and
the sound going to a file, and checks the window, the picture, fullscreen,
starting a game, sound, pause, the Controls screen, quitting and the saved
settings. The gamepad decoding is tested with recorded event sequences of
PS3, PS4, Xbox and plain USB pads. The code has two dependencies: `x11rb`
(with MIT-SHM and RENDER) and `libc`.

## License

MIT, see [LICENSE](LICENSE).
