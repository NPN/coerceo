/*
 * Copyright (C) 2017-2018 Ryan Huang
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use model::{BitBoard, Color, ColorMap, FieldCoord};

use self::OptionFieldCoord::*;

/// A wrapper enum representing a `FieldCoord` which may be invalid (i.e. one that is off the board).
/// Useful for keeping lookup table generation clean.
enum OptionFieldCoord {
    Some(FieldCoord),
    None,
}

pub const HEX_STARTING_POSITION: BitBoard = 0x1FF_FFFF_FFFF_FFFF;

#[cfg_attr(rustfmt, rustfmt_skip)]
/// Generate the Laurentius starting position.
pub fn generate_laurentius() -> ColorMap<BitBoard> {
    let mut white = 0;
    let mut black = 0;

    // (0, 0) is the only empty hex.
    // All other hexes have exactly two pieces on them in the starting position.
    let piece_locations = [
        (-2,  2, 0, 4),
        (-2,  1, 0, 3),
        (-2,  0, 3, 5),
        (-1,  2, 1, 4),
        (-1,  1, 0, 4),
        (-1,  0, 3, 5),
        (-1, -1, 2, 5),
        ( 0,  2, 1, 5),
        ( 0,  1, 1, 5),
        ( 0, -1, 2, 4),
        ( 0, -2, 2, 4),
        ( 1,  1, 2, 5),
        ( 1,  0, 0, 2),
        ( 1, -1, 1, 3),
        ( 1, -2, 1, 4),
        ( 2,  0, 0, 2),
        ( 2, -1, 0, 3),
        ( 2, -2, 1, 3),
    ];

    {
        let mut set_field = |coord: FieldCoord| match coord.color() {
            Color::White => white |= coord.to_bitboard(),
            Color::Black => black |= coord.to_bitboard(),
        };
        for &(x, y, f1, f2) in &piece_locations {
            set_field(FieldCoord::new(x, y, f1));
            set_field(FieldCoord::new(x, y, f2));
        }
    }
    ColorMap::new(white, black)
}

pub fn generate_edge_neighbors(color: Color) -> [BitBoard; 57] {
    let mut neighbors = [0; 57];

    for index in 0..57 {
        let coord = OptionFieldCoord::from_index(index, color);
        neighbors[index as usize] =
            fold_coords(&[coord.flip(), coord.shift_f(1), coord.shift_f(-1)]);
    }
    neighbors
}

pub fn generate_vertex_neighbors(color: Color) -> [BitBoard; 57] {
    let mut neighbors = [0; 57];

    for index in 0..57 {
        let coord = OptionFieldCoord::from_index(index, color);
        neighbors[index as usize] = fold_coords(&[
            coord.flip().shift_f(1),
            coord.flip().shift_f(-1),
            coord.shift_f(1).flip(),
            coord.shift_f(-1).flip(),
            coord.shift_f(2),
            coord.shift_f(-2),
        ]);
    }
    neighbors
}

pub fn generate_hex_mask() -> [BitBoard; 19] {
    let mut masks = [0; 19];
    let mut mask = 0b111;

    for hex in 0..19 {
        masks[hex] = mask;
        mask <<= 3;
    }
    masks
}

pub fn generate_hex_field_neighbors(color: Color) -> [BitBoard; 19] {
    let mut neighbors = [0; 19];

    let field_neighbor = |hex, f| OptionFieldCoord::from_hex_f(hex, f).flip();

    for hex in 0..19 {
        neighbors[hex as usize] = fold_coords(&match color {
            Color::White => [
                field_neighbor(hex, 0),
                field_neighbor(hex, 2),
                field_neighbor(hex, 4),
            ],
            Color::Black => [
                field_neighbor(hex, 1),
                field_neighbor(hex, 3),
                field_neighbor(hex, 5),
            ],
        });
    }
    neighbors
}

pub fn generate_removable_hex_combs() -> [BitBoard; 342] {
    let mut table = [0; 342];

    let bb_neighbor = |hex, f| OptionFieldCoord::from_hex_f(hex, f).flip().to_bitboard();

    for hex in 0..19 {
        let neighbors = [
            bb_neighbor(hex, 0),
            bb_neighbor(hex, 1),
            bb_neighbor(hex, 2),
            bb_neighbor(hex, 3),
            bb_neighbor(hex, 4),
            bb_neighbor(hex, 5),
        ];

        // This bitwise or combines bitboards of different "colors." However, since each neighbor is
        // on a different hex (i.e. in a different block of three bits), bits can never overlap each
        // other. Also, this is compared against the extant hex bitboard, so color doesn't matter.
        let mut triple = neighbors[0] | neighbors[1] | neighbors[2];
        let mut double = neighbors[0] | neighbors[1];
        let mut single = neighbors[0];

        for f in 0..6 {
            let index = hex as usize * 18 + f;
            table[index] = triple;
            table[index + 6] = double;
            table[index + 12] = single;

            triple ^= neighbors[f] | neighbors[(f + 3) % 6];
            double ^= neighbors[f] | neighbors[(f + 2) % 6];
            single ^= neighbors[f] | neighbors[(f + 1) % 6];
        }
    }
    table
}

fn fold_coords(coords: &[OptionFieldCoord]) -> BitBoard {
    coords.iter().fold(0, |acc, c| acc | c.to_bitboard())
}

impl OptionFieldCoord {
    fn from_index(index: u8, color: Color) -> Self {
        Some(FieldCoord::from_index(index, color))
    }
    fn from_hex_f(hex: u8, f: u8) -> Self {
        Some(FieldCoord::from_hex_f(hex, f))
    }
    fn shift_f(&self, n: i8) -> Self {
        assert!(-6 < n && n < 6);

        match *self {
            Some(coord) => Some(FieldCoord::new(
                coord.x,
                coord.y,
                (coord.f + (n + 6) as u8) % 6,
            )),
            None => None,
        }
    }
    /// Return the edge neighbor of this field that does not share its hex, i.e. "flip" this field
    /// over the boundary of its hex.
    fn flip(&self) -> Self {
        match *self {
            Some(coord) => {
                let (x, y) = match coord.f {
                    0 => (coord.x, coord.y + 1),
                    1 => (coord.x + 1, coord.y),
                    2 => (coord.x + 1, coord.y - 1),
                    3 => (coord.x, coord.y - 1),
                    4 => (coord.x - 1, coord.y),
                    5 => (coord.x - 1, coord.y + 1),
                    _ => unreachable!(),
                };
                let f = (coord.f + 3) % 6;

                if FieldCoord::is_valid_coord(x, y, f) {
                    Some(FieldCoord::new(x, y, f))
                } else {
                    None
                }
            }
            None => None,
        }
    }
    fn to_bitboard(&self) -> BitBoard {
        match *self {
            Some(coord) => coord.to_bitboard(),
            None => 0,
        }
    }
}
