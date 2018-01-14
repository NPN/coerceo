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

use model::{Color, FieldCoord, GameResult, HexCoord, Model, Move};
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
                        let (capture_count, mut fields_to_check) =
                            check_hexes(model, &selected.to_hex());
                        fields_to_check.append(&mut model.board.get_field_edge_neighbors(&clicked));
                        check_captures(model, &fields_to_check);

                        match model.turn {
                            Color::White => model.white_hexes += capture_count,
                            Color::Black => model.black_hexes += capture_count,
                        }

                        model.last_move = Move::Move(selected, clicked);
                        model.switch_turns();
                    }
                    clear_selection(model);
                    check_win(model);
                }
            } else if model.exchanging && model.board.is_piece_on_field(&clicked) {
                model.exchanging = false;
                model.board.remove_piece(&clicked);
                match model.turn {
                    Color::White => model.white_hexes -= 2,
                    Color::Black => model.black_hexes -= 2,
                }

                // Players don't collect hexes removed due to an exchange
                let (_, fields_to_check) = check_hexes(model, &clicked.to_hex());
                check_captures(model, &fields_to_check);

                model.last_move = Move::Exchange(clicked);
                model.switch_turns();
                check_win(model);
            } else {
                clear_selection(model);
            },
            Exchange => {
                if model.can_exchange() {
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

fn check_captures(model: &mut Model, fields_to_check: &[FieldCoord]) {
    for field in fields_to_check {
        if field.color() != model.turn && model.board.is_piece_on_field(field)
            && model
                .board
                .get_field_edge_neighbors(field)
                .into_iter()
                .all(|coord| model.board.is_piece_on_field(&coord))
        {
            model.board.remove_piece(field);
        }
    }
}

fn check_hexes(model: &mut Model, coord: &HexCoord) -> (u32, Vec<FieldCoord>) {
    let mut remove_count = 0;
    let mut fields = vec![];

    if model.board.remove_hex(coord) {
        remove_count += 1;
        for f in 0..6 {
            if let Some(neighbor) = model.board.get_hex_neighbor(coord, f) {
                let (removed, mut new_fields) = check_hexes(model, &neighbor);
                if remove_count == 0 {
                    let field = neighbor.to_field((f + 3) % 6);
                    if field.color() != model.turn {
                        fields.push(field);
                    }
                } else {
                    remove_count += removed;
                    fields.append(&mut new_fields);
                }
            }
        }
    }
    (remove_count, fields)
}
