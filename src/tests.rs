/*
 * Copyright (C) 2017-2019 Ryan Huang
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

use crate::model::{Board, GameType};

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

// All of the following perft results have not been verified by an external source. They only test
// for consistency with earlier versions of the program.

#[test]
fn laurentius_perft_4() {
    let counts = [48, 2304, 110304, 5280654];
    let board = Board::new(GameType::Laurentius, 2);

    for (i, &count) in counts.iter().enumerate() {
        assert_eq!(count, perft(&board, i as u8 + 1));
    }
}

#[test]
#[ignore]
fn laurentius_perft_5() {
    let counts = [48, 2304, 110304, 5280654, 254945184];
    let board = Board::new(GameType::Laurentius, 2);

    for (i, &count) in counts.iter().enumerate() {
        assert_eq!(count, perft(&board, i as u8 + 1));
    }
}

#[test]
#[ignore]
fn laurentius_perft_6() {
    let counts = [48, 2304, 110304, 5280654, 254945184, 12307984056];
    let board = Board::new(GameType::Laurentius, 2);

    for (i, &count) in counts.iter().enumerate() {
        assert_eq!(count, perft(&board, i as u8 + 1));
    }
}

#[test]
fn ocius_perft_5() {
    let counts = [26, 676, 17234, 435572, 10739924];
    let board = Board::new(GameType::Ocius, 2);

    for (i, &count) in counts.iter().enumerate() {
        assert_eq!(count, perft(&board, i as u8 + 1));
    }
}

#[test]
#[ignore]
fn ocius_perft_6() {
    let counts = [26, 676, 17234, 435572, 10739924, 262208752];
    let board = Board::new(GameType::Ocius, 2);

    for (i, &count) in counts.iter().enumerate() {
        assert_eq!(count, perft(&board, i as u8 + 1));
    }
}

#[test]
#[ignore]
fn ocius_perft_7() {
    let counts = [26, 676, 17234, 435572, 10739924, 262208752, 6252014770];
    let board = Board::new(GameType::Ocius, 2);

    for (i, &count) in counts.iter().enumerate() {
        assert_eq!(count, perft(&board, i as u8 + 1));
    }
}
