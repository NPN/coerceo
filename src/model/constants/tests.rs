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

use model::constants::*;
use model::Color;

#[test]
#[ignore]
fn edge_neighbors() {
    let neighbors = |color| {
        (0..57).map(move |index| {
            let coord = OptionFieldCoord::from_index(index, color);
            fold_coords(&[coord.flip(), coord.shift_f(1), coord.shift_f(-1)])
        })
    };

    assert!(
        EDGE_NEIGHBORS
            .0
            .white
            .iter()
            .map(|&x| x)
            .eq(neighbors(Color::White))
    );
    assert!(
        EDGE_NEIGHBORS
            .0
            .black
            .iter()
            .map(|&x| x)
            .eq(neighbors(Color::Black))
    );
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

    assert!(
        VERTEX_NEIGHBORS
            .0
            .white
            .iter()
            .map(|&x| x)
            .eq(neighbors(Color::White))
    );
    assert!(
        VERTEX_NEIGHBORS
            .0
            .black
            .iter()
            .map(|&x| x)
            .eq(neighbors(Color::Black))
    );
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

    assert!(
        HEX_FIELD_NEIGHBORS
            .0
            .white
            .iter()
            .map(|&x| x)
            .eq(neighbors(Color::White))
    );
    assert!(
        HEX_FIELD_NEIGHBORS
            .0
            .black
            .iter()
            .map(|&x| x)
            .eq(neighbors(Color::Black))
    );
}
