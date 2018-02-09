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

use model::{BitBoard, Color};

#[derive(Clone, Copy, PartialEq)]
struct FieldCoord {
    x: i8,
    y: i8,
    f: u8,
}

pub fn generate_edge_neighbors(color: Color) -> [BitBoard; 57] {
    let mut neighbors = [0; 57];

    for index in 0..57 {
        let coord = FieldCoord::from_index(index, color);
        neighbors[index as usize] =
            fold_coords(&[coord.flip(), coord.shift_f(1), coord.shift_f(-1)]);
    }
    neighbors
}

pub fn generate_vertex_neighbors(color: Color) -> [BitBoard; 57] {
    let mut neighbors = [0; 57];

    for index in 0..57 {
        let coord = FieldCoord::from_index(index, color);
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

    let field_neighbor = |hex, f| FieldCoord::from_hex_f(hex, f).flip();

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

    let bb_neighbor = |hex, f| FieldCoord::from_hex_f(hex, f).flip().to_bitboard();

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

fn fold_coords(coords: &[FieldCoord]) -> BitBoard {
    coords.iter().fold(0, |acc, &c| acc | c.to_bitboard())
}

impl FieldCoord {
    fn new(x: i8, y: i8, f: u8) -> FieldCoord {
        FieldCoord { x, y, f }
    }
    fn is_valid_coord(&self) -> bool {
        (self.x + self.y).abs() <= 2 && self.x.abs() <= 2 && self.y.abs() <= 2 && self.f < 6
    }
    fn from_index(index: u8, color: Color) -> FieldCoord {
        assert!(index < 57);

        let f = 2 * (index % 3) + match color {
            Color::White => 1,
            Color::Black => 0,
        };

        Self::from_hex_f(index / 3, f)
    }
    fn from_hex_f(hex: u8, f: u8) -> FieldCoord {
        assert!(hex < 19);
        assert!(f < 6);

        let hex = hex as i8 + match hex {
            0...2 => 2,
            3...15 => 3,
            16...18 => 4,
            _ => unreachable!(),
        };
        Self::new(hex % 5 - 2, hex / 5 - 2, f as u8)
    }
    fn to_bitboard(&self) -> BitBoard {
        if !self.is_valid_coord() {
            return 0;
        }
        let hex = 5 * (self.y + 2) + self.x + 2;
        let hex = hex as u8 - match hex {
            2...4 => 2,
            6...18 => 3,
            20...22 => 4,
            _ => unreachable!(),
        };

        1 << (hex * 3 + self.f / 2)
    }
    fn shift_f(&self, n: i8) -> FieldCoord {
        assert!(-6 < n && n < 6);

        Self::new(self.x, self.y, (self.f + (n + 6) as u8) % 6)
    }
    /// Return the edge neighbor of this field that does not share its hex, i.e. "flip" this field
    /// over the boundary of its hex.
    fn flip(&self) -> FieldCoord {
        let (x, y) = match self.f {
            0 => (self.x, self.y + 1),
            1 => (self.x + 1, self.y),
            2 => (self.x + 1, self.y - 1),
            3 => (self.x, self.y - 1),
            4 => (self.x - 1, self.y),
            5 => (self.x - 1, self.y + 1),
            _ => unreachable!(),
        };

        Self::new(x, y, (self.f + 3) % 6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_reflextivity() {
        for index in 0..57 {
            let white = FieldCoord::from_index(index as u8, Color::White);
            assert_eq!(index, white.to_bitboard().trailing_zeros());

            let black = FieldCoord::from_index(index as u8, Color::Black);
            assert_eq!(index, black.to_bitboard().trailing_zeros());
        }
    }
}
