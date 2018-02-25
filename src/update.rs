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
use model::{FieldCoord, Model, Move};
use view::Event;

pub fn update(model: &mut Model, event: Option<Event>) -> bool {
    let ai_should_move = |model: &Model| model.is_ai_turn() && !model.is_game_over();

    if let Some(event) = event {
        if event == Event::Quit {
            return false;
        }

        if ai_should_move(model) {
            use view::Event::*;
            match event {
                Click(_) | Exchange => return true,
                _ => {
                    let _ = model.ai_handle.as_ref().unwrap().stop_sender.send(());
                }
            }
        }

        handle_event(model, event);

        if ai_should_move(model) {
            model.ai_handle = Some(ai_move(model.board, 6, model.ai_handle.take()));
        }
    } else if ai_should_move(model) {
        if let Ok(mv) = model.ai_handle.as_ref().unwrap().move_receiver.try_recv() {
            if let Some(mv) = mv {
                model.ai_handle = None;
                model.board.apply_move(&mv);
                model.last_move = Some(mv);
            } else {
                unimplemented!("AI couldn't find a move");
            }
        }
    }
    true
}

fn handle_event(model: &mut Model, event: Event) {
    use view::Event::*;
    match event {
        Click(clicked) => if !model.is_game_over() {
            handle_click(model, clicked);
        },
        Exchange => if model.board.can_exchange() && !model.is_game_over() {
            model.exchanging = !model.exchanging;
            model.clear_selection();
        },
        NewGame(players) => *model = Model::new(players),
        Resign => {
            model.commit_state();
            model.board.resign();
        }
        Undo => model.undo_move(),
        Redo => model.redo_move(),
        Quit => unreachable!(),
    }
}

fn handle_click(model: &mut Model, clicked: FieldCoord) {
    match model.selected_piece {
        Some(selected) => {
            if clicked.color() != model.board.turn() || selected == clicked {
                model.clear_selection();
            } else if model.board.is_piece_on_field(&clicked) {
                model.available_moves = Some(model.board.available_moves_for_piece(&clicked));
                model.selected_piece = Some(clicked);
            } else {
                try_move(model, Move::move_from_field(&selected, &clicked));
                model.clear_selection();
            }
        }
        None => {
            if model.exchanging && try_move(model, Move::exchange_from_field(&clicked)) {
                model.exchanging = false;
            } else if !model.exchanging && clicked.color() == model.board.turn()
                && model.board.is_piece_on_field(&clicked)
            {
                model.available_moves = Some(model.board.available_moves_for_piece(&clicked));
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
        true
    } else {
        false
    }
}
