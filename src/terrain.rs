#[rustfmt::skip]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Terrain {
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
pub const TERRAIN: [Terrain; 256] = [
    // 0
    Unknown, DeepWater, Water, Shoals, Swamp, Grass, Brush, Desert,
    HeavyBrush, Trees, TropicalForest, Foothills, Mountains, HighPeaks,
    Foothills, Foothills,

    // 16
    SmallHut, CodexShrine, Keep, Village, Towne, Castle, Cave, Mine, Dungeon,
    Shrine, RuinedShrine, Lighthouse, Oasis, Bridge, Desert, Desert,

    // 32
    Road, Road, Road, Road, Road, Road, Road, Roof, Roof, CrystalSphere,
    LighthouseLight, HollowStump, PlowedPatch, Crops, Tree, Cactus,

    // 48
    Grass, Grass, Grass, Grass, Shoals, Shoals, Shoals, Shoals, Gargoyle,
    Castle, Castle, Castle, Castle, Castle, Castle, Castle,

    // 64
    WoodenPlanks, Codex, Mast, Rail, Cobble, Cobble, Pillar, Pier,
    WoodenPlanks, WoodenPlanks, ArrowSlit, Window, Rocks, StoneWall,
    SecretDoor, BrickWall,

    // 80
    Crenellations, Crenellations, Crenellations, Crenellations, Crenellations,
    Crenellations, Crenellations, Crenellations, Anvil, Spyglass, WindowShelf,
    PottedPlant, Bookshelf, Bookshelf, Guardian, Guardian,

    // 96
    River, River, River, River, River, River, River, River, River, River,
    Bridge, Bridge, River, River, River, River,

    // 112
    StrangeWall, StrangeWall, StrangeWall, StrangeWall, StrangeWall,
    StrangeWall, StrangeWall, StrangeWall, StrangeWall, StrangeWall,
    StrangeWall, StrangeWall, StrangeWall, StrangeWall, StrangeWall,
    StrangeWall,

    // 128
    Pendulum, Pendulum, Pendulum, Pendulum, Stocks, Manacles, Grate, Archway,
    Cannonballs, Grave, Gravestone, Rack, Trapdoor, Harpsichord, Guillotine,
    Lava,

    // 144
    Chair, Chair, Chair, Chair, Table, Table, Table, MagicDoor,
    MagicWindowDoor, Portcullis, TableWithFood, TableWithFood, TableWithFood,
    Mirror, MirrorReflection, BrokenMirror,

    // 160
    Sign, Well, HitchingPost, Logs, Marker, Desk, Barrel, Cask, VanityTable,
    Pitcher, Carpet, Bed, Bed, ChestOfDrawers, EndTable, Footlocker,

    // 176
    Torch, Torch, Brazier, Spit, Cannon, Cannon, Cannon, Cannon, Door,
    LockedDoor, WindowDoor, LockedWindowDoor, Fireplace, StreetLamp,
    Candelabrum, Stove,

    // 192
    Unknown, Unknown, Unknown, Unknown, Stairs, Stairs, Stairs, Stairs,
    Ladder, Ladder, Fence, Fence, Unknown, Unknown, Unknown, Unknown,

    // 208
    Wall, Wall, Wall, Wall, Waterfall, Waterfall, Waterfall, Waterfall,
    Fountain, Fountain, Fountain, Fountain, MoonGate, Desert, Flame,
    CollapsedDungeon,

    // 224
    Flagpole, Flagpole, Flagpole, Flagpole, Wall, Wall, Wall, Wall, Hourglass,
    Hourglass, Hourglass, Hourglass, Standard, Standard, Standard, Standard,

    // 240
    ProvisionerSign, GovernmentSign, ArmourySign, HealerSign, StableSign,
    GuildSign, InnSign, ApothecarySign, Unknown, ShipwrightSign,
    GrandfatherClock, GrandfatherClock, Bellows, Bellows, Wall, Darkness,
];
