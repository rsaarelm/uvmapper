# Ultima V combat screen level dungeon mapper

Reads game data from Ultima V game and builds topdown dungeon maps with the
special encounter rooms and corridor maps as seen during combat.

Have [Rust language tools](https://www.rust-lang.org/tools/install) installed
to use.

Set path to Ultima V game directory with data files in environment variable
`ULTIMA_V_PATH`, eg

    ULTIMA_V_PATH=~/Games/Ultima5 cargo run --release

See the [img/ subdirectory](img/) for all generated maps.

## Example results

![Covetous lvl 4](img/covetous-4.png)

Covetous level 4

![Shame lvl 8](img/shame-8.png)

Shame level 8

## TODO

- Show fields and doorways in dungeon blocks.
- Custom doorway alignment for tricky rooms in Destard.
- Optional custom offsets for the wraparound dungeon levels so that the
  room arrangements aren't cut by the wrap.
- Command-line interface for output directory, showing secret door triggers
  etc.
