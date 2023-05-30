use std::{
    collections::{HashMap, HashSet, VecDeque},
    env, fmt, fs, mem,
    path::PathBuf,
};

use clap::Parser;
use image::{ImageBuffer, Rgb};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

mod combat_map;
use combat_map::CombatMap;

mod terrain;
use terrain::{Terrain, TERRAIN};

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

// EGA color palette.
const EGA: [Rgb<u8>; 16] = [
    Rgb([0x00, 0x00, 0x00]),
    Rgb([0x00, 0x00, 0xAA]),
    Rgb([0x00, 0xAA, 0x00]),
    Rgb([0x00, 0xAA, 0xAA]),
    Rgb([0xAA, 0x00, 0x00]),
    Rgb([0xAA, 0x00, 0xAA]),
    Rgb([0xAA, 0x55, 0x00]),
    Rgb([0xAA, 0xAA, 0xAA]),
    Rgb([0x55, 0x55, 0x55]),
    Rgb([0x55, 0x55, 0xFF]),
    Rgb([0x55, 0xFF, 0x55]),
    Rgb([0x55, 0xFF, 0xFF]),
    Rgb([0xFF, 0x55, 0x55]),
    Rgb([0xFF, 0x55, 0xFF]),
    Rgb([0xFF, 0xFF, 0x55]),
    Rgb([0xFF, 0xFF, 0xFF]),
];

#[derive(Copy, Clone, Debug)]
pub enum Color {
    Black = 0,
    Navy,
    Green,
    Teal,
    Maroon,
    Purple,
    Olive,
    Silver,
    Gray,
    Blue,
    Lime,
    Aqua,
    Red,
    Fuchsia,
    Yellow,
    White,
}

use Color::*;

lazy_static! {
    static ref TILES: [[[Rgb<u8>; 16]; 16]; 512] = {
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

        // First four bytes are expected output length, skip those.
        let tiles =
            unpack_lzw(&fs::read(U5_PATH.join("TILES.16")).unwrap()[4..]);

        // Expand two 16-color pixels in each byte.
        let mut tiles: Vec<u8> =
            tiles.into_iter().flat_map(|b| [b >> 4, b & 0xf]).collect();

        // The game does some dynamic graphics tricks with the tiles, try to
        // replicate some here.

        // Recolor green grass into red dungeon dirt in all early terrain tiles.
        for t in 0..128 {
            // Except for swamp and some indoor decorations that have actual
            // green.
            if matches!(t, 4 | 91 | 92 | 93 | 94) {
                continue;
            }
            for i in t*256..(t+1)*256 {
                if tiles[i] == Green as u8 {
                    tiles[i] = Maroon as u8;
                }
            }
        }

        // Splice up (200) and down (201) ladders into an up/down ladder in
        // one of the nearby junk tiles (204).
        for i in 0..256 {
            // Top from up ladder
            if i < 128 {
                tiles[204*256 + i] = tiles[200*256 + i];
            } else {
                tiles[204*256 + i] = tiles[201*256 + i];
            }
        }

        // Convert to Rgb values.
        let tiles: Vec<Rgb<u8>> =
            tiles.into_iter().map(|b| EGA[b as usize]).collect();

        assert_eq!(tiles.len(), 16 * 16 * 512);
        let mut ret = [[[Rgb([0, 0, 0]); 16]; 16]; 512];
        for (i, b) in tiles.into_iter().enumerate() {
            ret[i / 256][(i / 16) % 16][i % 16] = b;
        }
        ret
    };
}

lazy_static! {
    static ref DUNGEONS: HashMap<&'static str, Dungeon> = {
        use DungeonKind::*;
        const DUNGEON_DATA: [(&str, DungeonKind); 8] = [
            ("Deceit", Prison),
            ("Despise", Cave),
            ("Destard", Cave),
            ("Wrong", Prison),
            ("Covetous", Prison),
            ("Shame", Mine),
            ("Hythloth", Mine),
            ("Doom", Cave),
        ];

        let dungeon_combat = fs::read(U5_PATH.join("DUNGEON.CBT")).unwrap();
        let mut rooms: Vec<CombatMap> = dungeon_combat.chunks(mem::size_of::<combat_map::CombatMapRaw>()).map(|c| bincode::deserialize(c).unwrap()).collect();

        // All dungeons except 2nd have 16 rooms. Insert dummy rooms for 2nd
        // dungeon to line up the array with the dungeons.
        rooms.splice(16..16, (0..16).map(|_| CombatMap::default()));

        let dungeons = fs::read(U5_PATH.join("DUNGEON.DAT")).unwrap();
        let dungeons: [[DungeonFloor; 8]; 8] =
            bincode::deserialize(&dungeons).unwrap();

        let mut ret = HashMap::new();
        for ((d, r), (name, kind)) in dungeons.into_iter().zip(rooms.chunks(16)).zip(DUNGEON_DATA) {
            ret.insert(name, Dungeon {
                kind,
                floors: d.into_iter().collect(),
                rooms: r.to_vec(),
            });
        }

        ret
    };
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(from = "u8")]
pub enum DungeonBlock {
    Corridor,
    UpLadder,
    DownLadder,
    UpDownLadder,

    Chest(u8),
    Fountain(u8),
    Trap(u8),

    OpenChest,

    Field(u8),

    Wall,
    SecretDoor,
    Door,

    Room(u8),

    Unknown,
}

impl fmt::Display for DungeonBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DungeonBlock::*;

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

impl From<u8> for DungeonBlock {
    fn from(b: u8) -> Self {
        use DungeonBlock::*;

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
struct DungeonFloor([[DungeonBlock; 8]; 8]);

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

#[derive(Copy, Clone, Debug)]
enum DungeonKind {
    Cave,
    Mine,
    Prison,
}

impl DungeonKind {
    fn wall_tile(self) -> usize {
        match self {
            DungeonKind::Prison => Terrain::BrickWall as usize,
            _ => Terrain::StoneWall as usize,
        }
    }

    fn doorway_tile(self) -> usize {
        match self {
            DungeonKind::Prison => Terrain::Door as usize,
            _ => Terrain::Grass as usize,
        }
    }

    fn floor_tile(self) -> usize {
        match self {
            DungeonKind::Prison => Terrain::Cobble as usize + 1,
            _ => Terrain::Grass as usize,
        }
    }
}

#[derive(Clone)]
struct Dungeon {
    pub kind: DungeonKind,
    pub floors: Vec<DungeonFloor>,
    pub rooms: Vec<CombatMap>,
}

#[derive(Default)]
struct TileData {
    pub tile: usize,
    pub monster: Option<usize>,
    pub is_trigger: bool,
    pub is_target: bool,
}

impl Dungeon {
    pub fn tile(&self, x: i32, y: i32, z: i32) -> TileData {
        const DARKNESS_TILE: usize = 255;
        use DungeonBlock::*;

        if z < 0 || z >= 8 {
            return Default::default();
        }

        if x < 0 || x >= 11 * 8 || y < 0 || y >= 11 * 8 {
            return Default::default();
        }

        // Which dungeon block is this?
        let (block_x, block_y) = ((x / 11) as usize, (y / 11) as usize);

        // Position within block.
        let (x, y) = ((x % 11) as u8, (y % 11) as u8);

        let block = self.floors[z as usize].0[block_y][block_x];

        if let Room(n) = block {
            let room = &self.rooms[n as usize];
            let tile = room.area[y as usize][x as usize] as usize;

            let monster = room.monsters.get(&[x, y]).cloned();
            let is_trigger = room.triggers.contains_key(&[x, y]);
            let is_target =
                room.triggers.iter().any(|(_, fx)| fx.contains_key(&[x, y]));

            return TileData {
                tile,
                monster,
                is_trigger,
                is_target,
            };
        }

        if matches!(block, Wall) {
            return TileData {
                tile: DARKNESS_TILE,
                ..Default::default()
            };
        }

        // Adjacent blocks.
        let (n, e, w, s) = (
            self.floors[z as usize].0[(block_y + 7) % 8][block_x],
            self.floors[z as usize].0[block_y][(block_x + 1) % 8],
            self.floors[z as usize].0[block_y][(block_x + 7) % 8],
            self.floors[z as usize].0[(block_y + 1) % 8][block_x],
        );

        // Walls in adjacent blocks.
        let (n_wall, e_wall, w_wall, s_wall) = (
            matches!(n, Wall),
            matches!(e, Wall),
            matches!(w, Wall),
            matches!(s, Wall),
        );

        // Distances from edges.
        let dw = x;
        let de = 10 - x;
        let dn = y;
        let ds = 10 - y;

        let vert_min = dn.min(ds);
        let horz_min = de.min(dw);

        // Edge walls.
        if (n_wall && dn == 0)
            || (e_wall && de == 0)
            || (w_wall && dw == 0)
            || (s_wall && ds == 0)
        {
            return TileData {
                tile: DARKNESS_TILE,
                ..Default::default()
            };
        }

        if (n_wall && dn == 1)
            || (e_wall && de == 1)
            || (w_wall && dw == 1)
            || (s_wall && ds == 1)
        {
            return TileData {
                tile: self.kind.wall_tile(),
                ..Default::default()
            };
        }

        // Center wall for force fields and doors.
        let is_center_wall =
            // Center cell, always include
            (x == 5 && y == 5)
            // Vertical wall
            || (n_wall && s_wall && (!e_wall || !w_wall) && x == 5 && (2..9).contains(&y))
            // Horizontal wall
            || (w_wall && e_wall && (!n_wall || !s_wall) && y == 5 && (2..9).contains(&x));

        // Doorways to rooms.

        // FIXME: This fails to align with rooms in Destard.
        // A fancier version could examine room map and align to open terrain
        // in it.
        if (matches!(n, Room(_)) && dn == 0 && de != 5)
            || (matches!(e, Room(_)) && de == 0 && dn != 5)
            || (matches!(w, Room(_)) && dw == 0 && dn != 5)
            || (matches!(s, Room(_)) && ds == 0 && de != 5)
        {
            return TileData {
                tile: self.kind.wall_tile(),
                ..Default::default()
            };
        }
        // Do dungeon-appropriate doors into the rooms.
        // XXX: Repetitious code
        if (matches!(n, Room(_)) && dn == 0 && de == 5)
            || (matches!(e, Room(_)) && de == 0 && dn == 5)
            || (matches!(w, Room(_)) && dw == 0 && dn == 5)
            || (matches!(s, Room(_)) && ds == 0 && de == 5)
        {
            return TileData {
                tile: self.kind.doorway_tile(),
                ..Default::default()
            };
        }

        // Corners.
        if vert_min.max(horz_min) == 0 {
            return TileData {
                tile: DARKNESS_TILE,
                ..Default::default()
            };
        }

        if vert_min.max(horz_min) == 1 {
            return TileData {
                tile: self.kind.wall_tile(),
                ..Default::default()
            };
        }

        let mut tile = self.kind.floor_tile();

        // Draw a wall around the door.
        if matches!(block, Door | SecretDoor) && is_center_wall {
            tile = self.kind.wall_tile();
        }

        // Center feature.
        if x == 5 && y == 5 {
            match block {
                UpLadder => tile = Terrain::Ladder as usize,
                DownLadder => tile = Terrain::Ladder as usize + 1,
                // NB: Using a generated tile.
                UpDownLadder => tile = 204,
                Chest(_) => tile = Terrain::Chest as usize,
                // Open chests don't seem to show up at all in the initial map data.
                OpenChest => tile = Terrain::Chest as usize,
                Fountain(_) => tile = Terrain::Fountain as usize,
                Trap(_) => tile = Terrain::Trapdoor as usize,
                Door => tile = Terrain::Door as usize,
                SecretDoor => tile = Terrain::SecretDoor as usize,
                _ => {}
            }
        }

        // Force field fills the whole middle area.
        if let Field(force) = block {
            let force = force as usize & 0x3;
            if is_center_wall {
                tile = 488 + force;
            }
        }

        TileData {
            tile,
            ..Default::default()
        }
    }

    pub fn pixel(&self, config: &Config, x: u32, y: u32, z: i32) -> Rgb<u8> {
        let (tile_x, tile_y) =
            (x.div_euclid(16) as i32, y.div_euclid(16) as i32);
        let (x, y) = (x.rem_euclid(16), y.rem_euclid(16));

        let data = self.tile(tile_x, tile_y, z);
        if data.tile == 0 {
            return EGA[0];
        }

        let tile_idx = if config.show_monsters {
            data.monster.unwrap_or(data.tile)
        } else {
            data.tile
        };

        let mut pixel = TILES[tile_idx][y as usize][x as usize];

        // Highlight trap tiles.
        if pixel == EGA[Black as usize] && config.show_secrets {
            if data.is_trigger && data.is_target {
                pixel = EGA[Olive as usize];
            } else if data.is_trigger {
                pixel = EGA[Green as usize];
            } else if data.is_target {
                pixel = EGA[Maroon as usize];
            }
        }

        pixel
    }

    pub fn draw_level_map(
        &self,
        config: &Config,
        level: i32,
    ) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        const SCREEN_WIDTH: i32 = 176;
        const SCREEN_HEIGHT: i32 = 176;

        assert!((0..8).contains(&level));

        // Mapping from unfolded block space (x, y) to the space of the 8x8
        // floor data array (u, v).
        let mut unfolded_blocks = HashMap::new();

        if config.unfold {
            use DungeonBlock::*;
            let floor = &self.floors[level as usize].0;

            // Unprocessed ground blocks. (physical_pos, logical_pos)
            let mut open_ground = VecDeque::new();
            // Unprocessed wall blocks next to ground. (physical_pos, logical_pos)
            let mut open_wall = VecDeque::new();
            // Processed blocks. Only stores physical positions.
            let mut closed = HashSet::new();

            // Find an open block as the starting point.
            'find_start: for y in 0..8 {
                for x in 0..8 {
                    if !matches!(floor[y][x], Wall) {
                        let x = x as i32;
                        let y = y as i32;
                        open_ground.push_back(((x, y), (x, y)));
                        break 'find_start;
                    }
                }
            }

            assert!(!open_ground.is_empty());

            'fill_regions: loop {
                // Fill the current open ground.
                while let Some(((u, v), (x, y))) = open_ground.pop_front() {
                    closed.insert((u, v));
                    unfolded_blocks.insert((x, y), (u, v));

                    for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                        let (x, y) = (x + dx, y + dy);
                        let (u, v) =
                            ((u + dx).rem_euclid(8), (v + dy).rem_euclid(8));
                        if closed.contains(&(u, v)) {
                            continue;
                        }
                        if matches!(floor[v as usize][u as usize], Wall) {
                            if !open_wall.contains(&((u, v), (x, y))) {
                                open_wall.push_back(((u, v), (x, y)));
                            }
                        } else {
                            if !open_ground.contains(&((u, v), (x, y))) {
                                open_ground.push_back(((u, v), (x, y)));
                            }
                        }
                    }
                }

                // Find the next region of open ground.
                while let Some(((u, v), (x, y))) = open_wall.pop_front() {
                    closed.insert((u, v));

                    for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                        let (x, y) = (x + dx, y + dy);
                        let (u, v) =
                            ((u + dx).rem_euclid(8), (v + dy).rem_euclid(8));
                        if closed.contains(&(u, v)) {
                            continue;
                        }
                        if matches!(floor[v as usize][u as usize], Wall) {
                            if !open_wall.contains(&((u, v), (x, y))) {
                                open_wall.push_back(((u, v), (x, y)));
                            }
                        } else {
                            if !open_ground.contains(&((u, v), (x, y))) {
                                open_ground.push_back(((u, v), (x, y)));
                                continue 'fill_regions;
                            }
                        }
                    }
                }

                // Must have processed the whole map to get here.
                assert_eq!(closed.len(), 64);

                break;
            }
        } else {
            for y in 0..8 {
                for x in 0..8 {
                    unfolded_blocks.insert((x, y), (x, y));
                }
            }
        }

        // Compute map bounds.
        let (mut x0, mut y0, mut x1, mut y1) =
            (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for (bx, by) in unfolded_blocks.keys() {
            let x = (bx) * SCREEN_WIDTH;
            let y = (by) * SCREEN_HEIGHT;

            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x + SCREEN_WIDTH);
            y1 = y1.max(y + SCREEN_HEIGHT);
        }

        image::ImageBuffer::from_fn(
            (x1 - x0) as u32,
            (y1 - y0) as u32,
            |x, y| {
                let (x, y) = (x as i32, y as i32);
                // Projected block position.
                let (bx, by) = (
                    (x0 + x).div_euclid(SCREEN_WIDTH),
                    (y0 + y).div_euclid(SCREEN_HEIGHT),
                );

                if let Some((bx, by)) = unfolded_blocks.get(&(bx, by)) {
                    self.pixel(
                        config,
                        (bx * SCREEN_WIDTH + x.rem_euclid(SCREEN_WIDTH)) as u32,
                        (by * SCREEN_HEIGHT + y.rem_euclid(SCREEN_HEIGHT))
                            as u32,
                        level,
                    )
                } else {
                    EGA[Black as usize]
                }
            },
        )
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// Hide monsters in combat rooms.
    #[arg(long)]
    hide_monsters: bool,
    /// Highlight trigger and target tiles in combat rooms.
    #[arg(long)]
    show_secrets: bool,
    /// Show the original 8x8 footprint instead of unfolding the dungeon.
    #[arg(long)]
    original_grid: bool,
}

struct Config {
    show_monsters: bool,
    show_secrets: bool,
    unfold: bool,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Config {
            show_monsters: !args.hide_monsters,
            show_secrets: args.show_secrets,
            unfold: !args.original_grid,
        }
    }
}

fn main() {
    let config = Config::from(Args::parse());

    for (name, dungeon) in &*DUNGEONS {
        for z in 0..8 {
            let map = dungeon.draw_level_map(&config, z);
            let filename = format!("{}-{}.png", name.to_lowercase(), z + 1);
            eprintln!("{}", filename);
            map.save(filename).unwrap();
        }
    }
}
