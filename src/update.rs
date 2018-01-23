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

use ai::ai_move;
use model::{Color, FieldCoord, GameResult, Model, Move};
use view::Event;

pub fn update(model: &mut Model, event: Option<Event>) {
    if let Some(event) = event {
        if model.is_ai_turn() {
            use view::Event::*;
            match event {
                Click(_) | Exchange => return,
                _ => {
                    let _ = model.ai_handle.as_ref().unwrap().stop_sender.send(());
                }
            }
        }

        handle_event(model, event);

        if model.is_ai_turn() {
            model.ai_handle = Some(ai_move(model.board, 5, model.ai_handle.take()));
        }
    } else if model.is_ai_turn() {
        if let Ok(mv) = model.ai_handle.as_ref().unwrap().move_receiver.try_recv() {
            if let Some(mv) = mv {
                model.ai_handle = None;
                model.board.apply_move(&mv);
                model.last_move = Some(mv);
                check_win(model);
            } else {
                unimplemented!("AI couldn't find a move");
            }
        }
    }
}

fn handle_event(model: &mut Model, event: Event) {
    use view::Event::*;
    match event {
        Click(clicked) => handle_click(model, clicked),
        Exchange => if model.board.can_exchange() {
            model.exchanging = !model.exchanging;
            model.clear_selection();
        },
        NewGame(players) => *model = Model::new(players),
        Resign => {
            model.commit_state();
            model.game_result = GameResult::Win(model.board.turn().switch());
        }
        Undo => model.undo_move(),
        Redo => model.redo_move(),
    }
}

fn handle_click(model: &mut Model, clicked: FieldCoord) {
    match model.selected_piece {
        Some(selected) => {
            if clicked.color() != model.board.turn() || selected == clicked {
                model.clear_selection();
            } else if model.board.is_piece_on_field(&clicked) {
                model.available_moves = Some(model.board.get_available_moves(&clicked));
                model.selected_piece = Some(clicked);
            } else {
                try_move(model, Move::Move(selected, clicked));
                model.clear_selection();
            }
        }
        None => {
            if model.exchanging && try_move(model, Move::Exchange(clicked)) {
                model.exchanging = false;
            } else if !model.exchanging && clicked.color() == model.board.turn()
                && model.board.is_piece_on_field(&clicked)
            {
                model.available_moves = Some(model.board.get_available_moves(&clicked));
                model.selected_piece = Some(clicked);
            }
        }
    }
}

fn try_move(model: &mut Model, mv: Move) -> bool {
    if model.board.can_apply_move(&mv) {
        model.commit_state();
        model.board.apply_move(&mv);
        model.last_move = Some(mv);
        check_win(model);
        true
    } else {
        false
    }
}

fn check_win(model: &mut Model) {
    if model.board.pieces(Color::White) == 0 {
        model.game_result = GameResult::Win(Color::White);
    } else if model.board.pieces(Color::Black) == 0 {
        model.game_result = GameResult::Win(Color::Black);
    }
}
