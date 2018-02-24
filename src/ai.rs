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

use model::{Board, Move, Outcome};

const INFINITY: i16 = 0x7FFF;
const NEG_INFINITY: i16 = -0x7FFF;
const LOSE: i16 = -0x4000;
const DRAW: i16 = 0;

pub struct AIHandle {
    pub move_receiver: Receiver<Option<Move>>,
    pub stop_sender: Sender<()>,
    handle: JoinHandle<()>,
}

pub fn ai_move(board: Board, depth: u8, prev_handle: Option<AIHandle>) -> AIHandle {
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

        // 2-ply iterative deepening
        let mut moves: Vec<(Move, i16)> = board
            .generate_moves()
            .into_iter()
            .map(|mv| {
                let mut new_board = board;
                new_board.apply_move(&mv);

                let score = -alphabeta_negamax(&new_board, NEG_INFINITY, INFINITY, 1);
                (mv, score)
            })
            .collect();

        moves.sort_by(|&(_, a), &(_, b)| b.cmp(&a));

        let mut max_score = NEG_INFINITY;
        let mut best_move = None;
        for (mv, _) in moves {
            if stop_receiver.try_recv().is_ok() {
                return;
            }

            let mut new_board = board;
            new_board.apply_move(&mv);

            let score = -alphabeta_negamax(&new_board, NEG_INFINITY, -max_score, depth - 1);
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

fn alphabeta_negamax(board: &Board, mut alpha: i16, beta: i16, depth: u8) -> i16 {
    match board.outcome() {
        Outcome::Draw => return DRAW,
        Outcome::Win(color) => {
            assert!(color != board.turn());
            // TODO: weight by depth to encourage shorter wins
            return LOSE;
        }
        _ => {}
    }

    if depth == 0 {
        evaluate(board)
    } else {
        let moves = board.generate_moves();
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

fn evaluate(board: &Board) -> i16 {
    use model::Color::*;

    let wp = 100 * i16::from(board.pieces(White));
    let bp = 100 * i16::from(board.pieces(Black));
    let wh = 50 * i16::from(board.hexes(White));
    let bh = 50 * i16::from(board.hexes(Black));

    match board.turn() {
        White => (wp + wh) - (bp + bh),
        Black => (bp + bh) - (wp + wh),
    }
}

pub fn perft(board: &Board, depth: u8) -> u32 {
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
