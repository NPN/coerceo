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

use model::{ColorMap, FieldCoord, Model, Move, Player};

use self::Event::*;

#[derive(PartialEq)]
pub enum Event {
    Click(FieldCoord),
    Exchange,
    NewGame(ColorMap<Player>),
    Resign,
    Undo,
    Redo,
    Quit,
}

pub fn update(model: &mut Model, event: Option<Event>) -> bool {
    if event == Some(Quit) {
        return false;
    }

    match model.current_player() {
        Player::Human => {
            if let Some(event) = event {
                handle_event(model, event);
            }
        }
        Player::Computer => {
            if let Some(event) = event {
                match event {
                    Click(_) | Exchange => {}
                    _ => {
                        model.ai.stop();
                        handle_event(model, event);
                        return true;
                    }
                }
            }

            if !model.is_game_over() {
                if model.ai.is_idle() {
                    model.ai.think(model.board, 6);
                }
                if let Some(mv) = model.ai.try_recv() {
                    model.board.apply_move(&mv);
                    model.last_move = Some(mv);
                }
            }
        }
    }
    true
}

fn handle_event(model: &mut Model, event: Event) {
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
