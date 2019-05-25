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

#![cfg(test)]

use crate::model::bitboard::BitBoard;
use crate::model::constants::*;
use crate::model::{Color, FieldCoord, HexCoord};

use self::OptionFieldCoord::*;

#[test]
#[ignore]
pub fn laurentius_starting_position() {
    let mut white = 0;
    let mut black = 0;

    // (0, 0) is the only empty hex.
    // All other hexes have exactly two pieces on them in the starting position.
    #[rustfmt::skip]
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
    assert_eq!(LAURENTIUS.hexes, (1 << 57) - 1);
    assert_eq!(LAURENTIUS.fields.white, white);
    assert_eq!(LAURENTIUS.fields.black, black);
}

#[test]
#[ignore]
pub fn ocius_starting_position() {
    let mut hexes = 0;
    let mut white = 0;
    let mut black = 0;

    // All hexes have exactly two pieces on them in the starting position.
    #[rustfmt::skip]
    let piece_locations = [
        (-1,  1, 1, 3),
        (-1,  0, 0, 2),
        ( 0,  1, 2, 4),
        ( 0,  0, 0, 3),
        ( 0, -1, 1, 5),
        ( 1,  0, 3, 5),
        ( 1, -1, 0, 4),
    ];

    {
        let mut set_field = |coord: FieldCoord| match coord.color() {
            Color::White => white |= coord.to_bitboard(),
            Color::Black => black |= coord.to_bitboard(),
        };
        for &(x, y, f1, f2) in &piece_locations {
            let hex = HexCoord::try_new(x, y).unwrap();
            hexes |= HEX_MASK[hex.to_index()];

            set_field(FieldCoord::new(x, y, f1));
            set_field(FieldCoord::new(x, y, f2));
        }
    }
    assert_eq!(OCIUS.hexes, hexes);
    assert_eq!(OCIUS.fields.white, white);
    assert_eq!(OCIUS.fields.black, black);
}

/// A wrapper enum representing a `FieldCoord` which may be invalid (i.e. one that is off the board).
/// Useful for keeping lookup table generation clean.
enum OptionFieldCoord {
    Some(FieldCoord),
    None,
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

fn fold_coords(coords: &[OptionFieldCoord]) -> BitBoard {
    coords.iter().fold(0, |acc, c| acc | c.to_bitboard())
}

#[test]
#[ignore]
fn edge_neighbors() {
    let neighbors = |color| {
        (0..57).map(move |index| {
            let coord = OptionFieldCoord::from_index(index, color);
            fold_coords(&[coord.flip(), coord.shift_f(1), coord.shift_f(-1)])
        })
    };

    assert!(EDGE_NEIGHBORS
        .0
        .white
        .iter()
        .map(|&x| x)
        .eq(neighbors(Color::White)));
    assert!(EDGE_NEIGHBORS
        .0
        .black
        .iter()
        .map(|&x| x)
        .eq(neighbors(Color::Black)));
}

#[test]
#[ignore]
fn vertex_neighbors() {
    let neighbors = |color| {
        (0..57).map(move |index| {
            let coord = OptionFieldCoord::from_index(index, color);
            fold_coords(&[
                coord.flip().shift_f(1),
                coord.flip().shift_f(-1),
                coord.shift_f(1).flip(),
                coord.shift_f(-1).flip(),
                coord.shift_f(2),
                coord.shift_f(-2),
            ])
        })
    };

    assert!(VERTEX_NEIGHBORS
        .0
        .white
        .iter()
        .map(|&x| x)
        .eq(neighbors(Color::White)));
    assert!(VERTEX_NEIGHBORS
        .0
        .black
        .iter()
        .map(|&x| x)
        .eq(neighbors(Color::Black)));
}

#[test]
#[ignore]
fn hex_field_neighbors() {
    let field_neighbor = |hex, f| OptionFieldCoord::from_hex_f(hex, f).flip();
    let neighbors = |color| {
        (0..19).map(move |hex| {
            fold_coords(&match color {
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
            })
        })
    };

    assert!(HEX_FIELD_NEIGHBORS
        .0
        .white
        .iter()
        .map(|&x| x)
        .eq(neighbors(Color::White)));
    assert!(HEX_FIELD_NEIGHBORS
        .0
        .black
        .iter()
        .map(|&x| x)
        .eq(neighbors(Color::Black)));
}

#[test]
#[ignore]
fn hex_mask() {
    let mut mask = 0b111;

    for hex in 0..19 {
        assert_eq!(HEX_MASK[hex], mask);
        mask <<= 3;
    }
}

#[test]
#[ignore]
fn removable_hex_combs() {
    let mut table = [0; 342];

    let neighbor = |hex, f| {
        OptionFieldCoord::from_hex_f(hex as u8, f as u8)
            .flip()
            .to_bitboard()
    };

    // The bitwise or's below combines bitboards of different "colors." However, since each neighbor
    // is on a different hex (i.e. in a different block of three bits), bits can never overlap each
    // other. Also, this is compared against the extant hex bitboard, so color doesn't matter.

    // Corner pieces
    for (f, &hex) in [7, 16, 18, 11, 2, 0].iter().enumerate() {
        let a = neighbor(hex, f);
        let b = neighbor(hex, (f + 1) % 6);
        let c = neighbor(hex, (f + 2) % 6);

        let index = hex * 18;

        table[index] = a | b | c;

        table[index + 1] = a | b;
        table[index + 2] = b | c;

        table[index + 3] = a;
        table[index + 4] = b;
        table[index + 5] = c;
    }

    // Edge pieces
    for (f, &hex) in [12, 17, 15, 6, 1, 3].iter().enumerate() {
        let a = neighbor(hex, f);
        let b = neighbor(hex, (f + 1) % 6);
        let c = neighbor(hex, (f + 2) % 6);
        let d = neighbor(hex, (f + 3) % 6);

        let index = hex * 18;

        table[index] = a | b | c;
        table[index + 1] = b | c | d;

        table[index + 2] = a | b;
        table[index + 3] = b | c;
        table[index + 4] = c | d;

        table[index + 5] = a;
        table[index + 6] = b;
        table[index + 7] = c;
        table[index + 8] = d;
    }

    // Center pieces
    for &hex in &[4, 5, 8, 9, 10, 13, 14] {
        let neighbors = [
            neighbor(hex, 0),
            neighbor(hex, 1),
            neighbor(hex, 2),
            neighbor(hex, 3),
            neighbor(hex, 4),
            neighbor(hex, 5),
        ];

        let mut triple = neighbors[0] | neighbors[1] | neighbors[2];
        let mut double = neighbors[0] | neighbors[1];
        let mut single = neighbors[0];

        for f in 0..6 {
            let index = hex * 18 + f;
            table[index] = triple;
            table[index + 6] = double;
            table[index + 12] = single;

            triple ^= neighbors[f] | neighbors[(f + 3) % 6];
            double ^= neighbors[f] | neighbors[(f + 2) % 6];
            single ^= neighbors[f] | neighbors[(f + 1) % 6];
        }
    }

    assert!(table.iter().eq(REMOVABLE_HEX_COMBS.iter()));
}
