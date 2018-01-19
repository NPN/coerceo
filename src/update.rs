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

        match event {
            Click(clicked) => match model.selected_piece {
                Some(selected) => {
                    if clicked.color() != model.board.turn() || selected == clicked {
                        model.clear_selection();
                    } else if model.board.is_piece_on_field(&clicked) {
                        model.available_moves = Some(model.board.get_available_moves(&clicked));
                        model.selected_piece = Some(clicked);
                    } else {
                        if model.board.can_move_piece(&selected, &clicked) {
                            model.commit_move();
                            model.board.move_piece(&selected, &clicked);
                            model.last_move = Move::Move(selected, clicked);
                            check_win(model);
                        }
                        model.clear_selection();
                    }
                }
                None => {
                    if model.exchanging && model.board.can_exchange_piece(&clicked) {
                        model.commit_move();
                        model.board.exchange_piece(&clicked);
                        model.last_move = Move::Exchange(clicked);
                        model.exchanging = false;
                        check_win(model);
                    } else if !model.exchanging && clicked.color() == model.board.turn()
                        && model.board.is_piece_on_field(&clicked)
                    {
                        model.available_moves = Some(model.board.get_available_moves(&clicked));
                        model.selected_piece = Some(clicked);
                    }
                }
            },
            NewGame => *model = Model::new(),
            Exchange => if model.board.can_exchange() {
                model.exchanging = !model.exchanging;
                model.clear_selection();
            },
            Resign => {
                model.commit_move();
                model.game_result = GameResult::Win(model.board.turn().switch());
            }
            Undo => model.undo_move(),
            Redo => model.redo_move(),
        }
    }
}

fn check_win(model: &mut Model) {
    if model.board.pieces(Color::White) == 0 {
        model.game_result = GameResult::Win(Color::White);
    } else if model.board.pieces(Color::Black) == 0 {
        model.game_result = GameResult::Win(Color::Black);
    }
}
