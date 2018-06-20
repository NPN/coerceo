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

use model::Board;

fn perft(board: &Board, depth: u8) -> u64 {
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
    // These counts have not been verified by an external source. They only test for consistency
    // with earlier versions of the program.
    let counts = [48, 2304, 110304, 5280654];
    let board = Board::new();

    for (i, &count) in counts.iter().enumerate() {
        assert_eq!(count, perft(&board, i as u8 + 1));
    }
}

#[test]
#[ignore]
fn perft_depth_5() {
    // These counts have not been verified by an external source. They only test for consistency
    // with earlier versions of the program.
    let counts = [48, 2304, 110304, 5280654, 254945184];
    let board = Board::new();

    for (i, &count) in counts.iter().enumerate() {
        assert_eq!(count, perft(&board, i as u8 + 1));
    }
}

#[test]
#[ignore]
fn perft_depth_6() {
    // These counts have not been verified by an external source. They only test for consistency
    // with earlier versions of the program.
    let counts = [48, 2304, 110304, 5280654, 254945184, 12307984056];
    let board = Board::new();

    for (i, &count) in counts.iter().enumerate() {
        assert_eq!(count, perft(&board, i as u8 + 1));
    }
}
