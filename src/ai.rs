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

use std::cmp;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use model::{Board, Move};

const INFINITY: i32 = 2_147_483_647;
const NEG_INFINITY: i32 = -2_147_483_647;
const LOSE: i32 = -1_073_741_824;
const DRAW: i32 = 0;

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

        let mut max_score = NEG_INFINITY;
        let mut best_move = None;
        for mv in generate_moves(&board) {
            if stop_receiver.try_recv().is_ok() {
                return;
            }

            let mut new_board = board;
            new_board.apply_move(&mv);

            let score = -alphabeta_negamax(&new_board, NEG_INFINITY, INFINITY, depth - 1);
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

fn alphabeta_negamax(board: &Board, mut alpha: i32, beta: i32, depth: u32) -> i32 {
    let moves = generate_moves(board);
    if moves.is_empty() {
        evaluate_empty(board)
    } else if depth == 0 {
        evaluate(board)
    } else {
        for mv in moves {
            let mut new_board = *board;
            new_board.apply_move(&mv);

            let score = -alphabeta_negamax(&new_board, -beta, -alpha, depth - 1);
            if score >= beta {
                return beta;
            } else if score > alpha {
                alpha = score;
            }
        }
        alpha
    }
}

// Assume the current player has at least one move to make
fn evaluate(board: &Board) -> i32 {
    use model::Color::*;
    let wp = board.pieces(White);
    let bp = board.pieces(Black);
    let wh = board.hexes(White);
    let bh = board.hexes(Black);

    // If neither side can capture the other's pieces, the game is drawn
    if wp == 1 && bp == 1 && (board.extant_hexes().len() as u32 + cmp::max(wh, bh) - 1 < 2) {
        return DRAW;
    }

    let wp = 100 * wp as i32;
    let bp = 100 * bp as i32;
    let wh = 50 * wh as i32;
    let bh = 50 * bh as i32;

    match board.turn() {
        White => (wp + wh) - (bp + bh),
        Black => (bp + bh) - (wp + wh),
    }
}

fn evaluate_empty(board: &Board) -> i32 {
    if board.pieces(board.turn()) == 0 {
        // TODO: weight by depth to encourage shorter wins
        LOSE
    } else {
        DRAW
    }

}

fn generate_moves(board: &Board) -> Vec<Move> {
    let turn = board.turn();
    // Ensure we don't let a player with zero pieces make exchange moves
    if board.pieces(turn) == 0 {
        return vec![];
    }

    let mut moves = vec![];
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
