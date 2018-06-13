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
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use model::ttable::{EvalType, TTable};
use model::{Board, Move, Outcome};

const NEG_INFINITY: i16 = -0x7FFF;
const LOSE: i16 = -0x4000;
// Small contempt factor to discourage draws
const DRAW: i16 = 1;

pub struct AI {
    status: Status,
    ttable: Arc<Mutex<TTable>>,
}

enum Status {
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
    pub fn new() -> Self {
        Self {
            status: Status::Idle,
            ttable: Arc::new(Mutex::new(TTable::new())),
        }
    }

    pub fn is_idle(&self) -> bool {
        match self.status {
            Status::Idle => true,
            Status::Thinking { .. } => false,
        }
    }

    pub fn stop(&mut self) {
        if let Status::Thinking {
            ref stop_signal, ..
        } = self.status
        {
            stop_signal.store(true, Ordering::Relaxed);
        }
        // Unconditionally assign because without NLL, we can't put this in the if let block above
        self.status = Status::Idle;
    }

    pub fn try_recv(&mut self) -> Option<Move> {
        use self::TryRecvError::*;

        let result;
        match self.status {
            Status::Idle => result = None,
            Status::Thinking { ref move_recv, .. } => match move_recv.try_recv() {
                Ok(mv) => result = Some(mv),
                Err(Empty) => result = None,
                Err(Disconnected) => panic!("Tried to receive move from disconnected sender"),
            },
        }

        // We can't set status in the Ok(mv) arm above without NLL
        if result.is_some() {
            self.status = Status::Idle;
        }
        result
    }

    pub fn think(&mut self, board: Board, board_list: Vec<Board>, depth: u8) {
        assert_ne!(depth, 0);

        let prev_status = mem::replace(&mut self.status, Status::Idle);

        let (move_sender, move_recv) = mpsc::channel();
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_signal_clone = stop_signal.clone();

        let ttable_mutex = self.ttable.clone();

        let handle = thread::spawn(move || {
            if let Status::Thinking {
                stop_signal,
                handle,
                ..
            } = prev_status
            {
                stop_signal.store(true, Ordering::Relaxed);
                handle
                    .join()
                    .expect("Old AI thread panicked when new AI thread joined on it");
            }

            use std::sync::TryLockError::*;
            let mut ttable = match ttable_mutex.try_lock() {
                Ok(table) => table,
                Err(Poisoned(_)) => panic!("Transposition table mutex is poisoned"),
                Err(WouldBlock) => {
                    panic!("Couldn't lock transposition table, is another AI thread still running?")
                }
            };
            ttable.inc_age();

            // Only take positions after the last irreversible move
            let mut board_list: Vec<_> = board_list
                .into_iter()
                .rev()
                .take_while(|b| b.vitals() == board.vitals())
                .collect();
            board_list.reverse();

            let mut moves: Vec<(Move, i16)> = board
                .generate_moves()
                .map(|mv| (mv, NEG_INFINITY))
                .collect();

            if moves.is_empty() {
                panic!("AI has no moves");
            }

            let mut pv = None;
            for depth in 0..depth {
                if stop_signal_clone.load(Ordering::Relaxed) {
                    return;
                }
                let mut max_score = NEG_INFINITY;
                // TODO: use the scores here are a kind of aspiration window?
                for pair in &mut moves {
                    let mut new_board = board;
                    new_board.apply_move(&pair.0);

                    let mut new_pv = vec![];

                    let score = -alphabeta_negamax(
                        &new_board,
                        &mut board_list,
                        &mut new_pv,
                        NEG_INFINITY,
                        -max_score,
                        depth,
                        &mut ttable,
                    );

                    if score > max_score {
                        max_score = score;
                        pv = Some(new_pv);
                    }
                    pair.1 = score;
                }
                moves.sort_by(|&(_, a), &(_, b)| b.cmp(&a));
                println!();
                println!("Depth {}: {:>6}", depth, moves[0].1);
                if let Some(ref mut pv) = pv {
                    pv.push(moves[0].0);
                    for mv in pv.iter().rev() {
                        println!("    {}", mv);
                    }
                }
            }
            println!();
            println!("---------------------");
            move_sender
                .send(moves[0].0)
                .expect("AI failed to send Move");
        });

        self.status = Status::Thinking {
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
    pv: &mut Vec<Move>,
    mut alpha: i16,
    mut beta: i16,
    depth: u8,
    ttable: &mut TTable,
) -> i16 {
    let mut set_pv = move |score, new_pv| {
        if score > alpha && score < beta {
            *pv = new_pv;
        }
    };

    let set_ttable = |ttable: &mut TTable, eval_type, score| {
        let zobrist = board.zobrist();
        ttable.set(zobrist, eval_type, depth, score);
    };

    match board.outcome() {
        Outcome::Draw => {
            // This only works because the draw Outcome does not consider draw by repetition
            set_ttable(ttable, EvalType::Exact, DRAW);
            set_pv(DRAW, vec![]);
            return DRAW;
        }
        Outcome::Win(color) => {
            assert_ne!(color, board.turn());
            // Weight score by depth to encourage shorter wins. The shorter the win, the greater
            // `depth` will be, and so the larger the score will be. This also encourages the AI to
            // prolong a loss.
            let score = LOSE - i16::from(depth);
            set_ttable(ttable, EvalType::Exact, score);
            set_pv(score, vec![]);
            return score;
        }
        Outcome::InProgress => {}
    }

    if let Some(entry) = ttable.get(board.zobrist()) {
        if board_list.len() >= 8 && board_list.iter().filter(|&&b| b == *board).count() >= 2 {
            set_pv(DRAW, vec![]);
            return DRAW;
        }

        if entry.zobrist == board.zobrist() && entry.depth == depth {
            match entry.eval_type {
                EvalType::Exact => {
                    // This will cut the PV short
                    // TODO: Store the best move in the table and get the PV from that?
                    set_pv(entry.score, vec![]);
                    return entry.score;
                }
                EvalType::Beta => {
                    if entry.score >= beta {
                        return entry.score;
                    }
                    beta = entry.score;
                }
            }
        }
    }

    if depth == 0 {
        let score = quiescence_search(board, alpha, beta);
        set_pv(score, vec![]);
        score
    } else {
        let mut best_score = NEG_INFINITY;
        let mut best_move = None;

        let mut new_pv = vec![];
        let moves = board.generate_moves();
        for mv in moves {
            let mut new_board = *board;
            new_board.apply_move(&mv);

            board_list.push(*board);
            let score = -alphabeta_negamax(
                &new_board,
                &mut board_list,
                &mut new_pv,
                -beta,
                -alpha,
                depth - 1,
                ttable,
            );
            board_list.pop();

            best_score = cmp::max(score, best_score);

            if score >= beta {
                set_ttable(ttable, EvalType::Beta, score);
                return beta;
            } else if score > alpha {
                alpha = score;
                best_move = Some(mv);
            }
        }
        set_ttable(ttable, EvalType::Exact, best_score);
        if let Some(mv) = best_move {
            new_pv.push(mv);
            set_pv(alpha, new_pv);
        }
        alpha
    }
}

// TODO: use ttable here?
fn quiescence_search(board: &Board, mut alpha: i16, beta: i16) -> i16 {
    let stand_pat = evaluate(board);
    if stand_pat >= beta {
        return beta;
    } else if alpha < stand_pat {
        alpha = stand_pat;
    }

    for mv in board.generate_captures() {
        let mut new_board = *board;
        new_board.apply_move(&mv);

        let score = quiescence_search(&new_board, -beta, -alpha);

        if score >= beta {
            return beta;
        } else if score > alpha {
            alpha = score;
        }
    }
    alpha
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
