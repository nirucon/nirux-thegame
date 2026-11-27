# NIRUX: the canon

This is the story bible for NIRUX and every game after it. Sequels follow
it so the hero, the world and the music stay the same from game to game.
Change it only on purpose, and change the games with it.

## The hero: NIRUX

NIRUX is the guardian the Core built and kept asleep inside itself, made
for one day only: the day the Core would fall. When NULL took the Core,
the Core spent its last power to wake NIRUX and throw it to the Outer Rim.

- **No face and no voice.** NIRUX never speaks and is never seen without
  the helmet. Other characters speak to NIRUX; the player is NIRUX.
- **No name but one.** Nobody knows who or what is inside the suit. Never
  answer that.
- **Epic, not jokey.** The tone is that of a 1980s game box: serious, a
  little mysterious, few words.

### How NIRUX looks

The silhouette must read at every size, from a few pixels on foot to a
full screen portrait.

| Part | Look | C64 colours |
| --- | --- | --- |
| Helmet | a pale dome with a ridge over the crown and a jaw guard | dark grey, grey, light grey, white |
| Visor | a dark plate with a wide, shallow chevron slit pointing down; the slit glows when NIRUX is awake | black plate, green halo, light green slit, a white glint sweeping across |
| Suit | blue armour, round shoulder guards in two plates with lit rims, a V collar | black, blue, light blue, cyan |
| Chest | a small diamond light: the Core's mark. It glows with the visor | light green, white centre |
| Scarf | red, wrapped round the neck, its forked end flying in the wind | brown, red, light red |

Rules: the visor is dark only when NIRUX sleeps or is gone. The scarf is
always there. Nothing else is added to the silhouette without changing
this file.

The code is `src/hero.rs`: `hero::portrait` draws the portrait at any size
and `hero::topdown` draws NIRUX on foot seen from above. A sequel copies
the file unchanged.

## The voice: ECHO

ECHO is the voice of the Core, the last of it that NULL could not reach.
ECHO briefs NIRUX before every sector, calm and brief, and is shown as
green text after `ECHO:`. ECHO never jokes and never says more than two
lines at a time.

## The enemy: NULL

NULL came out of the Static. It is never shown whole: it appears as static
tearing the picture, as the things it takes over, and as red text. In
NIRUX it held the Core and sealed the eight sectors. When NIRUX drove it
out, NULL was not destroyed. It fled down into the dark, where the deleted
things go.

## The world

- **The Core** keeps the eight sectors alive: The Outer Rim, Glass Dunes,
  Iron Jungle, The Shipyards, Ice Pylons, Burning City, The Gauntlet and
  the Core itself.
- **The Static** is the noise outside everything, where NULL came from.
- **The Undercroft** lies beneath the Core: the realm where deleted things
  go, old and dark. Terminated processes rise there as zombies, orphaned
  ones wander, and daemons rule. It is the setting of NIRUX II.

## The music

The hero's motif is the **call**: D, A, then up to the high D, followed by
a fall (`d5 a5 d6 | f6 d6 bb5` in the song notation). It is the core of
"Legend of Nirux" (the prologue) and comes back in every NIRUX game, in
the key and style of that game.

## The games

| Game | Title | Status |
| --- | --- | --- |
| I | NIRUX | released: a shooter in eight sectors |
| II | NIRUX II - NIGHT OF THE DAEMONS | planned, see [NIRUX-II.md](NIRUX-II.md) |

Sequels are titled `NIRUX II - SUBTITLE`, `NIRUX III - SUBTITLE` and so
on, and live in their own repositories as NIRUX extras.
