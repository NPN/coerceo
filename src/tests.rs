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

use model::{Board, Color, FieldCoord};

fn perft(board: &Board, depth: u8) -> u32 {
    if depth == 0 {
        1
    } else {
        let mut sum = 0;
        for mv in board.generate_moves() {
            let mut new_board = *board;
            new_board.apply_move(&mv);
            sum += perft(&new_board, depth - 1);
        }
        sum
    }
}

#[test]
fn perft_depth_4() {
    let counts = [48, 2304, 110304, 5280654];
    let board = Board::new();

    for (i, &count) in counts.iter().enumerate() {
        assert_eq!(count, perft(&board, i as u8 + 1));
    }
}

#[test]
fn fieldcoord_index_reflextivity() {
    for index in 0..57 {
        let white = FieldCoord::from_index(index as u8, Color::White);
        assert_eq!(index, white.to_index());

        let black = FieldCoord::from_index(index as u8, Color::Black);
        assert_eq!(index, black.to_index());
    }
}
