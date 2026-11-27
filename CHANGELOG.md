# Changelog

# NIRUX 0.2.0

The whole campaign.

- Five new sectors: The Shipyards (gantries, two capital ships, the
  Foundry), Ice Pylons (moving walls, switching fences, bombers, the
  Reactor), Burning City (rooftop snipers, trains, fires, the heli
  gunship), The Gauntlet (full speed, rock tunnels, the Warden Mk II and
  the Executor) and The NIRUX Core, which runs a space, an isometric and
  an on-foot stage in a row with a stage of the final boss at the end of
  each.
- Mid-bosses everywhere: gunships in the space sectors, the Sentinel in
  the isometric ones, tanks on foot. The script waits while one fights.
- Longer sectors (three to six minutes each), difficulty tuned per sector:
  Easy gentler, Hard harder.
- Iron Jungle is longer: temple ruins, a swamp, a plantation, a supply
  camp and a beach landing, and locked gates that open once the waves of
  attackers at them are beaten.
- On foot, holding fire keeps the aim, so the soldier can strafe.
- Every boss and final boss stage has a test that proves it can be
  destroyed on every difficulty, and the autopilot clears all eight
  sectors on every difficulty.
- The sides of the finale's isometric deck are drawn in clear colours.
- Hard is harder in space (aimed shots come as fans of three) and fairer on
  foot.
- An ending with the credits rolling.
- Seven new songs: Shipyards, Ice Pylons, Burning City, The Gauntlet, The
  Core, Heart of Nirux (the final boss) and Freedom (the ending).
- A Controls screen, from the title menu and the pause menu, with the keys,
  the gamepad buttons, a little pad and the name of the connected pad.
  Every sector's intro tells what its special does.
- Gamepads through Linux evdev: PS3, PS4, PS5, Xbox, 8BitDo and plain USB
  pads, found in sysfs so other input devices are never opened, plugged in
  any time, with the old joystick interface as a fallback. The menus work
  fully with a pad.
- Only the rows of the picture that changed are scaled and sent, which
  lowers the CPU use in menus and quiet scenes.
- The autopilot (attract mode and self test) plans its way over the
  isometric decks and walks the on-foot maps by path finding, and the self
  test covers all eight sectors.

# NIRUX 0.1.1

- The picture is scaled in the game and sent with MIT-SHM again by default;
  server-side RENDER scaling made an AMD laptop's X session crash and is now
  opt-in (NIRUX_RENDER=1).

# NIRUX 0.1.1

- Fix: on an AMD laptop (Xorg with glamor on amdgpu) the game made the
  graphics driver fault after about a minute, and the whole X session ended.
  The picture is now scaled in the game and sent with MIT-SHM, like the rest
  of NIRUX. The X server's RENDER scaling stays available with
  `NIRUX_RENDER=1`.

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
