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

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use model::{Board, Move};

pub struct AIHandle {
    pub move_receiver: Receiver<Option<Move>>,
    pub stop_sender: Sender<()>,
    handle: JoinHandle<()>,
}

pub fn ai_move(board: Board, depth: u32, prev_handle: Option<AIHandle>) -> AIHandle {
    assert!(depth != 0);

    let (move_sender, move_receiver) = mpsc::channel();
    let (stop_sender, stop_receiver) = mpsc::channel();

    let handle = thread::spawn(move || {
        if let Some(prev_handle) = prev_handle {
            // If send returns an error, the other thread has already terminated
            if prev_handle.stop_sender.send(()).is_ok() {
                prev_handle.handle.join().expect(
                    "Previous AI thread panicked while new AI thread was waiting for it to finish",
                );
            }
        }

        let mut max_score = i32::min_value();
        let mut best_move = None;
        for mv in generate_moves(&board) {
            if stop_receiver.try_recv().is_ok() {
                return;
            }

            let mut new_board = board;
            new_board.apply_move(&mv);

            let score = -negamax(&new_board, depth - 1);
            if score > max_score {
                max_score = score;
                best_move = Some(mv);
            }
        }
        move_sender.send(best_move).expect("AI failed to send Move");
    });

    AIHandle {
        move_receiver,
        stop_sender,
        handle,
    }
}

fn negamax(board: &Board, depth: u32) -> i32 {
    if depth == 0 {
        evaluate(board)
    } else {
        let moves = generate_moves(board);
        if moves.is_empty() {
            evaluate(board)
        } else {
            let mut max = i32::min_value();
            for mv in moves {
                let mut new_board = *board;
                new_board.apply_move(&mv);
                max = max.max(-negamax(&new_board, depth - 1));
            }
            max
        }
    }
}

fn evaluate(board: &Board) -> i32 {
    use model::Color::*;
    let wp = 100 * board.pieces(White) as i32;
    let bp = 100 * board.pieces(Black) as i32;

    let wh = 50 * board.hexes(White) as i32;
    let bh = 50 * board.hexes(Black) as i32;

    match board.turn() {
        White => (wp + wh) - (bp + bh),
        Black => (bp + bh) - (wp + wh),
    }
}

fn generate_moves(board: &Board) -> Vec<Move> {
    let mut moves = vec![];
    let turn = board.turn();
    let can_exchange = board.can_exchange();

    for hex in board.extant_hexes() {
        for f in 0..6 {
            let field = hex.to_field(f);
            if board.is_piece_on_field(&field) {
                if field.color() == turn {
                    moves.append(&mut board
                        .get_available_moves(&field)
                        .into_iter()
                        .map(|to| Move::Move(field, to))
                        .collect());
                } else if can_exchange {
                    moves.push(Move::Exchange(field));
                }
            }
        }
    }
    moves
}

pub fn perft(board: &Board, depth: u32) -> u32 {
    if depth == 0 {
        1
    } else {
        let mut sum = 0;
        for mv in generate_moves(board) {
            let mut new_board = *board;
            new_board.apply_move(&mv);
            sum += perft(&new_board, depth - 1);
        }
        sum
    }
}
