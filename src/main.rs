use std::{
    collections::HashMap,
    env, fmt,
    fs::{self, File},
    io::{prelude::*, BufReader},
    mem,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

fn unpack_lzw(mut bytes: &[u8]) -> Vec<u8> {
    let mut decoder = lzw::Decoder::new(lzw::LsbReader::new(), 8);
    let mut ret = Vec::new();
    loop {
        let (len, unpacked) = decoder.decode_bytes(bytes).unwrap();
        if len == 0 {
            break;
        }
        ret.extend_from_slice(unpacked);
        bytes = &bytes[len..];
    }
    ret
}

#[rustfmt::skip]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Terrain {
    DeepWater, Water, Shoals, Swamp, Grass, Brush, Desert, HeavyBrush, Trees,
    TropicalForest, Foothills, Mountains, HighPeaks,

    SmallHut, CodexShrine, Keep, Village, Towne, Castle, Cave, Mine, Dungeon,
    Shrine, RuinedShrine, Lighthouse, Oasis, Bridge,

    Road, Roof, CrystalSphere, LighthouseLight, HollowStump, PlowedPatch,
    Crops, Tree, Cactus,

    Gargoyle,

    WoodenPlanks, Codex, Mast, Rail, Cobble, Pillar, Pier, ArrowSlit, Window,
    Rocks, StoneWall, SecretDoor, BrickWall,

    Crenellations, Anvil, Spyglass, WindowShelf, PottedPlant, Bookshelf,
    Guardian,

    River,

    StrangeWall,

    Pendulum, Stocks, Manacles, Grate, Archway, Cannonballs, Grave,
    Gravestone, Rack, Trapdoor, Harpsichord, Guillotine, Lava,

    Chair, Table, MagicDoor, MagicWindowDoor, Portcullis, TableWithFood,
    Mirror, MirrorReflection, BrokenMirror,

    Sign, Well, HitchingPost, Logs, Marker, Desk, Barrel, Cask, VanityTable,
    Pitcher, Carpet, Bed, ChestOfDrawers, EndTable, Footlocker,

    Torch, Brazier, Spit, Cannon, Door, LockedDoor, WindowDoor,
    LockedWindowDoor, Fireplace, StreetLamp, Candelabrum, Stove,

    Stairs, Ladder, Fence,

    Waterfall, Fountain, MoonGate, Flame, CollapsedDungeon,

    Flagpole, Hourglass, Standard,

    ProvisionerSign, GovernmentSign, ArmourySign, HealerSign, StableSign,
    GuildSign, InnSign, ApothecarySign, ShipwrightSign, GrandfatherClock,
    Bellows, Wall, Darkness,

    Unknown,
}
use Terrain::*;

impl Into<char> for Terrain {
    fn into(self) -> char {
        match self {
            DeepWater => '≋',
            Water => '≈',
            Shoals => '~',
            River => '~',
            Swamp => ',',
            Grass => '.',
            Road => '.',
            Brush => '%',
            Desert => '.',
            HeavyBrush => '%',
            Trees => '%',
            TropicalForest => '%',
            Foothills => '^',
            Mountains => '^',
            HighPeaks => '^',
            Lava => '&',

            Cobble => '.',

            WoodenPlanks => '.',
            Rocks => '*',
            StoneWall => '*',
            SecretDoor => '#',
            BrickWall => '#',
            StrangeWall => '#',
            Wall => '#',
            Mast => '0',

            Ladder => '<',
            Stairs => '<',

            Window => '+',
            ArrowSlit => '+',
            LockedWindowDoor => '|',

            Darkness => ' ',

            // TODO, fill in more
            _ => '?',
        }
    }
}

#[rustfmt::skip]
const TERRAIN: [Terrain; 256] = [
    Unknown, DeepWater, Water, Shoals, Swamp, Grass, Brush, Desert,
    HeavyBrush, Trees, TropicalForest, Foothills, Mountains, HighPeaks,
    Foothills, Foothills,

    SmallHut, CodexShrine, Keep, Village, Towne, Castle, Cave, Mine, Dungeon,
    Shrine, RuinedShrine, Lighthouse, Oasis, Bridge, Desert, Desert,

    Road, Road, Road, Road, Road, Road, Road, Roof, Roof, CrystalSphere,
    LighthouseLight, HollowStump, PlowedPatch, Crops, Tree, Cactus,

    Grass, Grass, Grass, Grass, Shoals, Shoals, Shoals, Shoals, Gargoyle,
    Castle, Castle, Castle, Castle, Castle, Castle, Castle,

    WoodenPlanks, Codex, Mast, Rail, Cobble, Cobble, Pillar, Pier,
    WoodenPlanks, WoodenPlanks, ArrowSlit, Window, Rocks, StoneWall,
    SecretDoor, BrickWall,

    Crenellations, Crenellations, Crenellations, Crenellations, Crenellations,
    Crenellations, Crenellations, Crenellations, Anvil, Spyglass, WindowShelf,
    PottedPlant, Bookshelf, Bookshelf, Guardian, Guardian,

    River, River, River, River, River, River, River, River, River, River,
    Bridge, Bridge, River, River, River, River,

    StrangeWall, StrangeWall, StrangeWall, StrangeWall, StrangeWall,
    StrangeWall, StrangeWall, StrangeWall, StrangeWall, StrangeWall,
    StrangeWall, StrangeWall, StrangeWall, StrangeWall, StrangeWall,
    StrangeWall,

    Pendulum, Pendulum, Pendulum, Pendulum, Stocks, Manacles, Grate, Archway,
    Cannonballs, Grave, Gravestone, Rack, Trapdoor, Harpsichord, Guillotine,
    Lava,

    Chair, Chair, Chair, Chair, Table, Table, Table, MagicDoor,
    MagicWindowDoor, Portcullis, TableWithFood, TableWithFood, TableWithFood,
    Mirror, MirrorReflection, BrokenMirror,

    Sign, Well, HitchingPost, Logs, Marker, Desk, Barrel, Cask, VanityTable,
    Pitcher, Carpet, Bed, Bed, ChestOfDrawers, EndTable, Footlocker,

    Torch, Torch, Brazier, Spit, Cannon, Cannon, Cannon, Cannon, Door,
    LockedDoor, WindowDoor, LockedWindowDoor, Fireplace, StreetLamp,
    Candelabrum, Stove,

    Unknown, Unknown, Unknown, Unknown, Stairs, Stairs, Stairs, Stairs,
    Ladder, Ladder, Fence, Fence, Unknown, Unknown, Unknown, Unknown,

    Wall, Wall, Wall, Wall, Waterfall, Waterfall, Waterfall, Waterfall,
    Fountain, Fountain, Fountain, Fountain, MoonGate, Desert, Flame,
    CollapsedDungeon,

    Flagpole, Flagpole, Flagpole, Flagpole, Wall, Wall, Wall, Wall, Hourglass,
    Hourglass, Hourglass, Hourglass, Standard, Standard, Standard, Standard,

    ProvisionerSign, GovernmentSign, ArmourySign, HealerSign, StableSign,
    GuildSign, InnSign, ApothecarySign, Unknown, ShipwrightSign,
    GrandfatherClock, GrandfatherClock, Bellows, Bellows, Wall, Darkness,
];

#[derive(Serialize, Deserialize, Debug)]
struct CombatMapRaw {
    row_0: [u8; 11],
    new_tiles: [u8; 8],
    pad_0: [u8; 13],

    row_1: [u8; 11],
    player_x_east: [u8; 6],
    player_y_east: [u8; 6],
    pad_1: [u8; 9],

    row_2: [u8; 11],
    player_x_west: [u8; 6],
    player_y_west: [u8; 6],
    pad_2: [u8; 9],

    row_3: [u8; 11],
    player_x_south: [u8; 6],
    player_y_south: [u8; 6],
    pad_3: [u8; 9],

    row_4: [u8; 11],
    player_x_north: [u8; 6],
    player_y_north: [u8; 6],
    pad_4: [u8; 9],

    row_5: [u8; 11],
    monsters: [u8; 16],
    pad_5: [u8; 5],

    row_6: [u8; 11],
    monsters_x: [u8; 16],
    pad_6: [u8; 5],

    row_7: [u8; 11],
    monsters_y: [u8; 16],
    pad_7: [u8; 5],

    row_8: [u8; 11],
    trigger_x: [u8; 8],
    trigger_y: [u8; 8],
    pad_8: [u8; 5],

    row_9: [u8; 11],
    change_0_x: [u8; 8],
    change_0_y: [u8; 8],
    pad_9: [u8; 5],

    row_10: [u8; 11],
    change_1_x: [u8; 8],
    change_1_y: [u8; 8],
    pad_10: [u8; 5],
}

struct Trigger {
    pos: [u8; 2],
    target_1: [u8; 2],
    target_2: [u8; 2],
    new_tile: u8,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(from = "CombatMapRaw")]
struct CombatMap {
    area: [[u8; 11]; 11],
    player_east: [[u8; 2]; 6],
    player_west: [[u8; 2]; 6],
    player_south: [[u8; 2]; 6],
    player_north: [[u8; 2]; 6],

    monsters: Vec<(u8, [u8; 2])>,

    triggers: Vec<([u8; 2], Vec<(u8, [u8; 2])>)>,
}

fn merge_coords<const N: usize>(xs: &[u8; N], ys: &[u8; N]) -> [[u8; 2]; N] {
    let mut ret: [[u8; 2]; N] = unsafe { mem::zeroed() };
    for i in 0..N {
        ret[i][0] = xs[i];
        ret[i][1] = ys[i];
    }
    ret
}

impl From<CombatMapRaw> for CombatMap {
    fn from(data: CombatMapRaw) -> Self {
        let mut ret = Self::default();
        for (y, row) in [
            data.row_0,
            data.row_1,
            data.row_2,
            data.row_3,
            data.row_4,
            data.row_5,
            data.row_6,
            data.row_7,
            data.row_8,
            data.row_9,
            data.row_10,
        ]
        .into_iter()
        .enumerate()
        {
            ret.area[y] = row;
        }

        ret.player_east =
            merge_coords(&data.player_x_east, &data.player_y_east);
        ret.player_west =
            merge_coords(&data.player_x_west, &data.player_y_west);
        ret.player_south =
            merge_coords(&data.player_x_south, &data.player_y_south);
        ret.player_north =
            merge_coords(&data.player_x_north, &data.player_y_north);

        for (i, &m) in data.monsters.iter().enumerate() {
            if m != 0 {
                ret.monsters
                    .push((m, [data.monsters_x[i], data.monsters_y[i]]));
            }
        }

        for (i, &t) in data.new_tiles.iter().enumerate() {
            if t != 0 {
                let trigger = [data.trigger_x[i], data.trigger_y[i]];

                let p1 = [data.change_0_x[i], data.change_0_y[i]];
                let p2 = [data.change_1_x[i], data.change_1_y[i]];
                let mut fx = vec![(t, p1)];
                if p2 != p1 {
                    fx.push((t, p2));
                }

                ret.triggers.push((trigger, fx));
            }
        }

        ret
    }
}

impl fmt::Display for CombatMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..11 {
            for x in 0..11 {
                let terrain = TERRAIN[self.area[y][x] as usize];
                let c: char = terrain.into();
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(from = "u8")]
pub enum DungeonTile {
    Corridor,
    UpLadder,
    DownLadder,
    UpDownLadder,

    Chest(u8),
    // TODO: Trap, status
    Fountain(u8),
    // TODO: Type
    Trap(u8),

    OpenChest,

    Field(u8),

    Wall,
    SecretDoor,
    Door,

    Room(u8),

    Unknown,
}

impl fmt::Display for DungeonTile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DungeonTile::*;

        let c = match self {
            Corridor => '.',
            UpLadder => '<',
            DownLadder => '>',
            UpDownLadder => '↔',
            Chest(_) => '$',
            Fountain(_) => '{',
            Trap(_) => '^',
            OpenChest => '$',
            Field(_) => '*',
            Wall => '#',
            SecretDoor => '+',
            Door => '|',
            Room(n) => char::from_digit(*n as u32, 16).unwrap(),
            Unknown => '?',
        };
        write!(f, "{}", c)
    }
}

impl From<u8> for DungeonTile {
    fn from(b: u8) -> Self {
        use DungeonTile::*;

        match b >> 4 {
            0 => Corridor,
            1 => UpDownLadder,
            2 => DownLadder,
            3 => UpDownLadder,
            4 => Chest(b & 0xf),
            5 => Fountain(b & 0xf),
            6 => Trap(b & 0xf),
            7 => OpenChest,
            8 => Field(b & 0xf),
            10 => Room(b & 0xf),
            11 => Wall,
            12 => Wall,
            13 => SecretDoor,
            14 => Door,
            15 => Room(b & 0xf),
            _ => Unknown,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
struct DungeonFloor([[DungeonTile; 8]; 8]);

impl fmt::Display for DungeonFloor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..8 {
            for x in 0..8 {
                write!(f, "{}", self.0[y][x])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

// Bitpos: &[u8], usize bit offse

fn lzw_decode(data: &[u8]) -> Vec<u8> {
    struct BitCursor<'a> {
        data: &'a [u8],
        bit_offset: usize,
    }

    impl<'a> BitCursor<'a> {
        pub fn read(&mut self, nbits: usize) -> u32 {
            debug_assert!(nbits > 0 && nbits < 16);
            let mut n = *self.data.get(0).unwrap_or(&0) as u32
                + (*self.data.get(1).unwrap_or(&0) as u32)
                << 8 + (*self.data.get(2).unwrap_or(&0) as u32)
                << 16;
            n >>= self.bit_offset;

            self.bit_offset += nbits;
            while self.bit_offset >= 8 {
                self.bit_offset -= 8;
                self.data = &self.data[1..];
            }

            n & ((1 << nbits) - 1)
        }
    }

    todo!();
}

fn main() {
    let path: PathBuf = env::var("ULTIMA_V_PATH")
        .expect("Set environment variable ULTIMA_V_PATH to point to data files")
        .into();
    if !fs::metadata(path.join("BRIT.CBT")).map_or(false, |m| m.is_file()) {
        eprintln!("Invalid Ultima V path {:?}", path);
        return;
    }

    let dungeon_combat = fs::read(path.join("DUNGEON.CBT")).unwrap();
    let n = dungeon_combat.len() / std::mem::size_of::<CombatMapRaw>();
    for c in dungeon_combat.chunks(mem::size_of::<CombatMapRaw>()) {
        let map: CombatMap = bincode::deserialize(c).unwrap();
        println!("{}", map);
    }

    // 2nd dungeon has no combat rooms, all other ones have 16.

    let dungeons = fs::read(path.join("DUNGEON.DAT")).unwrap();
    let dungeons: [[DungeonFloor; 8]; 8] =
        bincode::deserialize(&dungeons).unwrap();

    for d in 0..8 {
        for f in 0..8 {
            println!("{}", dungeons[d][f]);
        }
        println!("--------\n");
    }

    // First four bytes are expected output length, skip.
    let tiles = unpack_lzw(&fs::read(path.join("TILES.16")).unwrap()[4..]);
    let tiles: Vec<u8> =
        tiles.into_iter().flat_map(|b| [b >> 4, b & 0xf]).collect();

    for y in 0..(512 * 16) {
        for x in 0..16 {
            let mut b = tiles[y * 16 + x];
            if b == 0 {
                print!(" ");
            } else {
                print!("{}", char::from_digit(b as u32, 16).unwrap());
            }
        }
        println!();
    }
}
