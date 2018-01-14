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

use model::{Color, GameResult, Model, Move};
use view::Event;

pub fn update(model: &mut Model, event: Option<Event>) {
    if let Some(event) = event {
        use view::Event::*;

        if event == NewGame {
            *model = Model::new();
            return;
        } else if model.game_result != GameResult::InProgress {
            return;
        }

        match event {
            Click(clicked) => if model.turn == clicked.color() {
                if model.board.is_piece_on_field(&clicked) {
                    if model.selected_piece.as_ref() == Some(&clicked) {
                        clear_selection(model);
                    } else {
                        model.available_moves = Some(model.board.get_available_moves(&clicked));
                        model.selected_piece = Some(clicked);
                    }
                } else if let Some(selected) = model.selected_piece.take() {
                    if model.board.move_piece(&selected, &clicked) {
                        model.last_move = Move::Move(selected, clicked);
                        model.switch_turns();
                    }
                    clear_selection(model);
                    check_win(model);
                }
            } else if model.exchanging && model.board.exchange_piece(&clicked) {
                model.exchanging = false;

                model.last_move = Move::Exchange(clicked);
                model.switch_turns();
                check_win(model);
            } else {
                clear_selection(model);
            },
            Exchange => {
                if model.board.can_exchange(&model.turn) {
                    model.exchanging = !model.exchanging;
                    clear_selection(model);
                }
            }
            Resign => {
                model.game_result = match model.turn {
                    Color::Black => GameResult::WhiteWin,
                    Color::White => GameResult::BlackWin,
                }
            }
            _ => {}
        }
    }
}

fn clear_selection(model: &mut Model) {
    model.selected_piece = None;
    model.available_moves = None;
}

fn check_win(model: &mut Model) {
    if model.board.white_pieces() == 0 {
        model.game_result = GameResult::BlackWin;
    } else if model.board.black_pieces() == 0 {
        model.game_result = GameResult::WhiteWin;
    }
}
