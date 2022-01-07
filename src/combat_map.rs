use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CombatMapRaw {
    // The native structure is messy,
    // <https://wiki.ultimacodex.com/wiki/Ultima_V_internal_formats>.
    // Convert it to the cleaner and more idomatic `CombatMap` when
    // deserializing.
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

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(from = "CombatMapRaw")]
pub struct CombatMap {
    pub area: [[u8; 11]; 11],
    pub player_east: [[u8; 2]; 6],
    pub player_west: [[u8; 2]; 6],
    pub player_south: [[u8; 2]; 6],
    pub player_north: [[u8; 2]; 6],

    pub monsters: HashMap<[u8; 2], usize>,

    pub triggers: HashMap<[u8; 2], HashMap<[u8; 2], u8>>,
}

fn merge_coords<const N: usize>(xs: &[u8; N], ys: &[u8; N]) -> [[u8; 2]; N] {
    let mut ret: [[u8; 2]; N] = unsafe { std::mem::zeroed() };
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
                let p = [data.monsters_x[i], data.monsters_y[i]];
                //debug_assert!(!ret.monsters.contains_key(&p));
                ret.monsters.insert(p, m as usize + 256);
            }
        }

        for (i, &t) in data.new_tiles.iter().enumerate() {
            if t != 0 {
                let trigger = [data.trigger_x[i], data.trigger_y[i]];
                //debug_assert!(!ret.triggers.contains_key(&trigger));

                let p1 = [data.change_0_x[i], data.change_0_y[i]];
                let p2 = [data.change_1_x[i], data.change_1_y[i]];

                ret.triggers
                    .insert(trigger, [(p1, t), (p2, t)].into_iter().collect());
            }
        }

        ret
    }
}

impl fmt::Display for CombatMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..11 {
            for x in 0..11 {
                let terrain = crate::TERRAIN[self.area[y][x] as usize];
                let c: char = terrain.into();
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
