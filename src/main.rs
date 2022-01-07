use std::{env, fmt, fs, mem, path::PathBuf};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

mod combat_map;
use combat_map::CombatMap;

mod terrain;
use terrain::TERRAIN;

// TODO: Colorize black differently in triggerable room cells?

// Dungeon corridor
//
// ##.......##  C: Center object (ladder etc.)
// ##.......##  d: Filled with wall if door
// ##.......##  .: "red grass" in cave/mine, gray hexagon floor (69) in dungeon
// ##.......##  #: Wall in dungeon, rock wall in mine/cave
// ##.......##
// ##d..C..d##  Side walls are shaped according to adjacent dungeon walls.
// ##.......##
// ##.......##  Grate tile for pit in floor.
// ##.......##
// ##.......##
// ##.......##
//
//
// ##.......##
// ###.....###  Seems like a door to the north can show up like this ingame.
// ...........  Might take artistic liberties with this since the door tile
// ...........  is still a full dungeon chunk.
// ...........
// ...........
// ...........
// ...........
// ...........
// ##.......##
// ##.......##

lazy_static! {
    static ref U5_PATH: PathBuf = {
        let path: PathBuf = env::var("ULTIMA_V_PATH")
            .expect(
                "Set environment variable ULTIMA_V_PATH to point to data files",
            )
            .into();
        if !fs::metadata(path.join("BRIT.CBT")).map_or(false, |m| m.is_file()) {
            eprintln!("Invalid Ultima V path {:?}", path);
            std::process::exit(1);
        }
        path
    };
}

lazy_static! {
    static ref TILES: [[[u8; 16]; 16]; 512] = {
        // TODO: Figure out which tile is used for the dungeon dirt ground
        // tiles and recolorize it here.

        // First four bytes are expected output length, skip those.
        let tiles =
            unpack_lzw(&fs::read(U5_PATH.join("TILES.16")).unwrap()[4..]);
        let tiles: Vec<u8> =
            tiles.into_iter().flat_map(|b| [b >> 4, b & 0xf]).collect();
        assert_eq!(tiles.len(), 16 * 16 * 512);
        let mut ret = [[[0; 16]; 16]; 512];
        for (i, b) in tiles.into_iter().enumerate() {
            ret[i / 256][(i / 16) % 16][i % 16] = b;
        }
        ret
    };
}

lazy_static! {
    // TODO: Figure out the order of the names, make this HashMap<String,
    // Dungeon>.
    static ref DUNGEONS: Vec<Dungeon> = {
        let dungeon_combat = fs::read(U5_PATH.join("DUNGEON.CBT")).unwrap();
        let mut rooms: Vec<CombatMap> = dungeon_combat.chunks(mem::size_of::<combat_map::CombatMapRaw>()).map(|c| bincode::deserialize(c).unwrap()).collect();

        // All dungeons except 2nd have 16 rooms. Insert dummy rooms for 2nd
        // dungeon to line up the array with the dungeons.
        rooms.splice(16..16, (0..16).map(|_| CombatMap::default()));

        let dungeons = fs::read(U5_PATH.join("DUNGEON.DAT")).unwrap();
        let dungeons: [[DungeonFloor; 8]; 8] =
            bincode::deserialize(&dungeons).unwrap();

        let mut ret = Vec::new();
        for (d, r) in dungeons.into_iter().zip(rooms.chunks(16)) {
            ret.push(Dungeon {
                floors: d.into_iter().collect(),
                rooms: r.iter().cloned().collect(),
            });
        }

        ret
    };
}

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

#[derive(Clone)]
struct Dungeon {
    pub floors: Vec<DungeonFloor>,
    pub rooms: Vec<CombatMap>,
}

// TODO: Pixel sampler for dungeon floors, needs Dungeon struct for context.

fn main() {
    let dungeon_combat = fs::read(U5_PATH.join("DUNGEON.CBT")).unwrap();
    for c in dungeon_combat.chunks(mem::size_of::<combat_map::CombatMapRaw>()) {
        let map: CombatMap = bincode::deserialize(c).unwrap();
        println!("{}", map);
    }

    // 2nd dungeon has no combat rooms, all other ones have 16.

    let dungeons = fs::read(U5_PATH.join("DUNGEON.DAT")).unwrap();
    let dungeons: [[DungeonFloor; 8]; 8] =
        bincode::deserialize(&dungeons).unwrap();

    for d in 0..8 {
        for f in 0..8 {
            println!("{}", dungeons[d][f]);
        }
        println!("--------\n");
    }

    for t in TILES.iter() {
        for y in 0..16 {
            for x in 0..16 {
                let b = t[y][x];
                if b == 0 {
                    print!(" ");
                } else {
                    print!("{}", char::from_digit(b as u32, 16).unwrap());
                }
            }
            println!();
        }
    }
}
