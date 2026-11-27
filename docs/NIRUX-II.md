# NIRUX II - NIGHT OF THE DAEMONS (concept)

A design concept for the second NIRUX game: a gothic action platformer in
the spirit of the 1980s castle and graveyard games, set in the NIRUX world
and drawn in the same sixteen C64 colours. Everything is original: the
story, the stages, the enemies, the mechanics, the art and the music. The
canon in [LORE.md](LORE.md) applies.

## The story

NULL fled down into the Undercroft, the realm beneath the Core where the
deleted things go. There it found what nothing else wanted: terminated
processes, orphaned programs, files nobody remembered. NULL raises them.
The dead processes walk again as zombies, the daemons that once served the
system now serve NULL, and every night the cron bells ring and the
Undercroft wakes.

ECHO, weakened, can follow NIRUX only so far down. NIRUX goes alone into
the dark to end NULL for good.

The tone stays epic: rain, lightning, ruins, bells, and one silent hero
with a glowing visor walking into it.

## How it plays

A side-scrolling platformer with running, jumping and shooting, built for
breaks like the first game: stages of five to eight minutes, checkpoints,
and every stage reached stays open in level select. The campaign is about
45 to 60 minutes straight through.

- **Weight and commitment.** Jumps follow a firm arc with a little control
  in the air, so every jump is a decision. Ladders, crumbling stairs,
  moving platforms and swinging chains.
- **The scarf is your health.** NIRUX's scarf burns shorter with every
  hit: three hits and it is gone, the next one costs a life. Relics at
  shrines mend it. The player always sees how much is left, on the hero
  and not on a bar.
- **The visor sees the hidden.** Holding the special lights the visor
  like a lamp: ghost files and hidden doors show up in its beam, and it
  drains slowly. A new use for the canon's glowing visor.
- **Weapons found in the stages.** The main bolt is thrown in a straight
  line; pickups swap it for an arcing hammer of light, a ricochet disc
  that bounces off walls, or a short chain whip that cuts in an arc.
  Taking a hit never takes the weapon.
- **The cron bells.** On some stages bells ring on a timer and a wave of
  wraiths sweeps through. A bell tolls three times before it happens, so
  the player can find cover.

## The stages

| Stage | Name | The idea |
| --- | --- | --- |
| 1 | The Process Graveyard | rain and lightning over the graves of terminated programs; zombies rise from the ground; the first boss is the Gravedigger |
| 2 | The Swap Marsh | fog, sinking platforms over black water, swarms of orphans; the Bog Daemon |
| 3 | The Archive Cathedral | a vertical climb past stained glass and bell ropes, ghost files that pass through walls; the Choir of the Deleted |
| 4 | The Cron Tower | gears, pendulums and platforms that move on the clock, the bells at their worst; the Clockwork Warden |
| 5 | The Frozen Cache | ice caverns with slippery floors and falling icicles; the Frost Wyrm |
| 6 | The Kernel Crypt | lava below, collapsing bridges, daemon knights; the Daemon Lord |
| 7 | NULL's Keep | the castle at the bottom of the Undercroft, all the stage ideas at once, and NULL at last |

A recurring stranger, **the Reaper**, the old process that once cleared
away the dead, appears in several stages. It hunts the zombies, and
sometimes NIRUX. Whether it is friend or foe is the game's open question
until stage 6.

## The enemies

| Enemy | What it does |
| --- | --- |
| Zombie process | rises from a grave, walks, keeps walking until destroyed |
| Orphan | small and fast, comes in swarms, flees when hit |
| Ghost file | floats through walls; visible only in the visor's light on some stages |
| Daemon | winged, swoops and throws fire |
| Daemon knight | armoured, blocks shots from the front |
| Wraith | arrives with the cron bells in a wave across the screen |
| Fork bomb | splits in two when hit, then again |

## Look and sound

- **Pixels:** 320x200 in the C64 palette as before, a parallax sky of
  two or three layers, lightning flashes that light the whole palette, and
  large hand-drawn bosses.
- **NIRUX** is a side-view sprite with the canon's colours: pale helmet,
  green chevron visor, blue suit, red scarf that flies behind when running.
- **Music:** gothic and minor key on the same three-voice synthesiser:
  organ-like pulse chords, harpsichord-like plucked arpeggios, a tolling
  bell, and the hero's call from "Legend of Nirux" returning in a dark
  key. One song per stage, a boss theme, a NULL theme and an ending.

## Building it

- A new repository, `nirux-ii`, installed as a NIRUX extra like the first
  game (`nirux extras install nirux-ii`).
- It starts from this game's engine: the window and scaling, the
  framebuffer and font, the synthesiser and song notation, the sound
  output, the gamepads and the input. `src/hero.rs` is copied unchanged.
- New: a tile map engine with scrolling, slopes, ladders and moving
  platforms; side-view sprites for the hero and enemies; the stages.
- Releases in steps: 0.1.0 with the first two stages playable and the
  engine done, then two stages a release until NULL's Keep.
