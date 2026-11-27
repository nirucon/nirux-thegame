//! Pixel art drawn for NIRUX, as text: every character is the C64 colour
//! of one pixel (hex digit) or a dot for transparent. Bigger things (the
//! capital ship, the bosses, the isometric world, the soldiers) are drawn
//! by code in the game modules.
use crate::fb::Sprite;
use std::sync::OnceLock;

pub struct Art {
    pub ship: Sprite,
    pub wisp: [Sprite; 2],
    pub dart: Sprite,
    pub hornet: [Sprite; 2],
    pub cannon: Sprite,
    pub tree: Sprite,
    pub palm: Sprite,
    pub bush: Sprite,
    pub crate_: Sprite,
    pub rock: Sprite,
    pub logo: Vec<&'static str>,
}

const SHIP: &[&str] = &[
    ".........bbb..............",
    "........bcfcb.............",
    ".........bcffb............",
    "....bbb...bcfffbbb........",
    "...bcccbbbbcfffffccbb.....",
    "..bcfffccccfff1ee3ccfcb...",
    ".bcfffffffffff1ee33ffffcc1",
    "..bcccccccfffffcccccccbb..",
    "...bbbbbbbcfffcbbbbbb.....",
    "..........bcfffbbb........",
    ".........bcffb............",
    "........bcfcb.............",
    ".........bbb..............",
];

const WISP_A: &[&str] = &[
    "....4......4....",
    ".....4....4.....",
    "....4a4444a4....",
    "..44aaa11aaa44..",
    ".4aaaa1aaaaaaa4.",
    "44a7aaaaaaaa7a44",
    ".4aaaaaaaaaaaa4.",
    "..444aaaaaa444..",
    "....4.4..4.4....",
];

const WISP_B: &[&str] = &[
    "...4........4...",
    "....4......4....",
    "....4a4444a4....",
    "..44aaa11aaa44..",
    ".4aaaa1aaaaaaa4.",
    "44a1aaa77aaa1a44",
    ".4aaaaaaaaaaaa4.",
    "..444aaaaaa444..",
    "...4..4..4..4...",
];

const DART: &[&str] = &[
    "..........bb2...",
    ".......bbc22a...",
    "..1bbccccc2aa22b",
    "7aaffffffffccccb",
    "..1bbccccc2aa22b",
    ".......bbc22a...",
    "..........bb2...",
];

const HORNET_A: &[&str] = &[
    "...d.......d...",
    "....d.....d....",
    "..55d55555d55..",
    ".5ddd55555ddd5.",
    "5d1dd55555dd1d5",
    "5ddd5577755ddd5",
    ".5dd5557555dd5.",
    "..5d5555555d5..",
    "...55..5..55...",
    "..5....5....5..",
    ".5...........5.",
];

const HORNET_B: &[&str] = &[
    "..d.........d..",
    "...d.......d...",
    "..55d55555d55..",
    ".5ddd55555ddd5.",
    "5d1dd55555dd1d5",
    "5ddd5577755ddd5",
    ".5dd5557555dd5.",
    "..5d5555555d5..",
    "...55..5..55...",
    "...5...5...5...",
    "...5.......5...",
];

const CANNON: &[&str] = &[
    "....bbbbbbbb....",
    "..bbccccccccbb..",
    ".bcffffffffffcb.",
    "bcf11fffffffffcb",
    "bcffffffffffffcb",
    "bbccccccccccccbb",
    ".bbbbbbbbbbbbbb.",
];

const TREE: &[&str] = &[
    ".....000000.......",
    "...00dd5dd500.....",
    "..0dd1dd5dd5d0....",
    ".0dd1dd55d555d0...",
    ".0d1dd5dd5d55550..",
    "0dddd5dd5555d5b0..",
    "0d5d55dd555555b0..",
    "0dd55d555d55b5b0..",
    "05d55555555b5bb0..",
    "0555d5555b5bbbb0..",
    ".05555b5bbbbbb0...",
    ".0b55bbbbbbbb00...",
    "..00bbbbbbb00.....",
    "....0000000.......",
];

const PALM: &[&str] = &[
    "....00.....00.....",
    "...0dd0...0dd0....",
    "0...0dd0.0dd0..00.",
    "d0...05d0d50..0dd0",
    "0dd0..05d50.00dd0.",
    ".0ddd00599005550..",
    "..00555909555b00..",
    ".00dd5d999d55dd0..",
    "0dd5000999000dd50.",
    "0d50..05d50..05d0.",
    ".00..0dd0d50..00..",
    ".....0d0.0d0......",
    "......0...0.......",
];

const BUSH: &[&str] = &[
    "...0000.....",
    ".00dd5d00...",
    "0d5dd5d5d0..",
    "0dd5d555b50.",
    ".055d555bb0.",
    "..00bbbb00..",
];

const CRATE: &[&str] = &[
    "9999999999999",
    "988888888888b",
    "98b8888888b8b",
    "988b88888b88b",
    "9888b888b888b",
    "98888b8b8888b",
    "988888b88888b",
    "98888b8b8888b",
    "9888b888b888b",
    "988b88888b88b",
    "98b8888888b8b",
    "988888888888b",
    "bbbbbbbbbbbbb",
];

const ROCK: &[&str] = &[
    "....bbbbb....",
    "..bbcccccbb..",
    ".bccfccccccb.",
    "bccfcccccbccb",
    "bcccccbcccccb",
    "bccbcccccccbb",
    ".bbccccccbbb.",
    "...bbbbbbb...",
];

/// The NIRUX logo letters on a grid; every `#` becomes a block of the
/// raster bar colours.
const LOGO: &[&str] = &[
    "###...###..#####..########...###...###..###...###",
    "####..###...###...###...###..###...###...###.###.",
    "#####.###...###...###...###..###...###....#####..",
    "###.#####...###...########...###...###.....###...",
    "###..####...###...###.###....###...###....#####..",
    "###...###...###...###..###...###...###...###.###.",
    "###...###..#####..###...###...#######...###...###",
];

pub fn get() -> &'static Art {
    static A: OnceLock<Art> = OnceLock::new();
    A.get_or_init(|| Art {
        ship: Sprite::parse(SHIP),
        wisp: [Sprite::parse(WISP_A), Sprite::parse(WISP_B)],
        dart: Sprite::parse(DART),
        hornet: [Sprite::parse(HORNET_A), Sprite::parse(HORNET_B)],
        cannon: Sprite::parse(CANNON),
        tree: Sprite::parse(TREE),
        palm: Sprite::parse(PALM),
        bush: Sprite::parse(BUSH),
        crate_: Sprite::parse(CRATE),
        rock: Sprite::parse(ROCK),
        logo: LOGO.to_vec(),
    })
}

/// The 32x32 window icon.
pub fn icon_rows() -> Vec<String> {
    let mut rows = vec![String::from("................................"); 32];
    for (y, r) in SHIP.iter().enumerate() {
        let line = format!("...{r}...");
        rows[y + 9] = format!("{:.<32}", line);
    }
    rows
}

#[cfg(test)]
mod tests {
    #[test]
    fn art_parses() {
        let a = super::get();
        assert_eq!(a.ship.w, 26);
        assert!(a.logo.iter().all(|r| r.len() == a.logo[0].len()));
        assert_eq!(super::icon_rows().len(), 32);
    }
}
