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

use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use model::{Board, Move, Outcome};

const INFINITY: i16 = 0x7FFF;
const NEG_INFINITY: i16 = -0x7FFF;
const LOSE: i16 = -0x4000;
const DRAW: i16 = 0;

pub enum AI {
    Idle,
    // Either the AI thread is running, or there is a move waiting to be received
    Thinking {
        move_recv: Receiver<Move>,
        // We store and load this atomic with Ordering::Relaxed. It *should* be fine because it
        // doesn't interact with any other atomics--all we want to do is tell the AI thread to stop
        // searching for a move
        stop_signal: Arc<AtomicBool>,
        handle: JoinHandle<()>,
    },
}

impl AI {
    pub fn new() -> AI {
        AI::Idle
    }

    pub fn is_idle(&self) -> bool {
        match self {
            AI::Idle => true,
            AI::Thinking { .. } => false,
        }
    }

    pub fn stop(&mut self) {
        if let AI::Thinking { stop_signal, .. } = self {
            stop_signal.store(true, Ordering::Relaxed);
            *self = AI::Idle;
        }
    }

    pub fn try_recv(&mut self) -> Option<Move> {
        use self::TryRecvError::*;

        match self {
            AI::Idle => None,
            AI::Thinking { move_recv, .. } => match move_recv.try_recv() {
                Ok(mv) => {
                    *self = AI::Idle;
                    Some(mv)
                }
                Err(Empty) => None,
                Err(Disconnected) => panic!("Tried to receive move from disconnected sender"),
            },
        }
    }

    pub fn think(&mut self, board: Board, board_list: Vec<Board>, depth: u8) {
        assert_ne!(depth, 0);

        let prev_ai = mem::replace(self, AI::Idle);

        let (move_sender, move_recv) = mpsc::channel();
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_signal_clone = stop_signal.clone();

        let handle = thread::spawn(move || {
            if let AI::Thinking {
                stop_signal,
                handle,
                ..
            } = prev_ai
            {
                stop_signal.store(true, Ordering::Relaxed);
                handle
                    .join()
                    .expect("Old AI thread panicked when new AI thread joined on it");
            }

            // Only take positions after the last irreversible move
            let mut board_list: Vec<_> = board_list
                .into_iter()
                .rev()
                .take_while(|b| b.vitals() == board.vitals())
                .collect();
            board_list.reverse();

            // 2-ply iterative deepening
            let mut moves: Vec<(Move, i16)> = board
                .generate_moves()
                .into_iter()
                .map(|mv| {
                    let mut new_board = board;
                    new_board.apply_move(&mv);

                    let score =
                        -alphabeta_negamax(&new_board, &mut board_list, NEG_INFINITY, INFINITY, 1);
                    (mv, score)
                })
                .collect();

            moves.sort_by(|&(_, a), &(_, b)| b.cmp(&a));

            let mut max_score = NEG_INFINITY;
            let mut best_move = None;
            for (mv, _) in moves {
                if stop_signal_clone.load(Ordering::Relaxed) {
                    return;
                }

                let mut new_board = board;
                new_board.apply_move(&mv);

                let score = -alphabeta_negamax(
                    &new_board,
                    &mut board_list,
                    NEG_INFINITY,
                    -max_score,
                    depth - 1,
                );
                if score > max_score {
                    max_score = score;
                    best_move = Some(mv);
                }
            }

            match best_move {
                Some(mv) => move_sender.send(mv).expect("AI failed to send Move"),
                None => panic!("AI failed to find a move"),
            }
        });

        *self = AI::Thinking {
            move_recv,
            stop_signal,
            handle,
        }
    }
}

fn alphabeta_negamax(
    board: &Board,
    // This list does not include the current board
    mut board_list: &mut Vec<Board>,
    mut alpha: i16,
    beta: i16,
    depth: u8,
) -> i16 {
    match board.outcome() {
        Outcome::Draw => return DRAW,
        Outcome::Win(color) => {
            assert_ne!(color, board.turn());
            // TODO: weight by depth to encourage shorter wins
            return LOSE;
        }
        Outcome::InProgress => {}
    }

    if depth == 0 {
        evaluate(board)
    } else {
        let moves = board.generate_moves();
        for mv in moves {
            let mut new_board = *board;
            new_board.apply_move(&mv);

            board_list.push(*board);
            let score = -alphabeta_negamax(&new_board, &mut board_list, -beta, -alpha, depth - 1);
            board_list.pop();

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
