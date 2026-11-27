# Changelog

# NIRUX 0.1.0

The first playable demo: three of the eight sectors, one of each kind of
play.

- Sector 1, The Outer Rim: a horizontal space shooter with fighter waves,
  an asteroid field, a capital ship to take apart section by section, and
  the Warden.
- Sector 2, Glass Dunes: isometric flight with altitude and a shadow,
  walls, energy fences, towers, turrets, and the Monolith.
- Sector 3, Iron Jungle: a top-down run and gun on foot with riflemen,
  grenadiers, gun nests, a river crossing, a tank, grenades, and the
  Gatekeeper.
- Title screen with a chrome logo, raster bars and a sine scroller; level
  select, Easy, Normal and Hard, options (CRT scanlines, fullscreen, music
  and effects volume, echo), high scores with name entry, pause, sector
  clear tally, game over, and an attract mode with a self-playing demo.
- A SID-style synthesiser (three music voices and two effect voices,
  resonant filter, PWM, sync, ring modulation, ADSR) with seven original
  songs and the sound effects. Sound through pw-cat, pacat or aplay.
- A 320x200 picture in the C64 palette, scaled by whole numbers (by the X
  server through RENDER where it can) with a C64 border around it in a
  colour per sector; fullscreen by default, F11 to switch.
- Keyboard and Linux joystick input; settings, progress and high scores
  saved in the XDG folders.
- `--selftest`, `--render-wav`, `--demo` and an Xvfb end-to-end test.
