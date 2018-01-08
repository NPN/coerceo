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

use model::{Color, FieldCoord, HexCoord, Model, Move};
use view::Event;

pub fn update(model: &mut Model, event: Option<Event>) {
    if let Some(event) = event {
        use view::Event::*;
        match event {
            Click(field) => if model.turn == field.color() {
                if model.board.is_piece_on_field(&field) {
                    if model.selected_piece.as_ref() == Some(&field) {
                        clear_selection(model);
                    } else {
                        model.available_moves = Some(model.board.get_available_moves(&field));
                        model.selected_piece = Some(field);
                    }
                } else if let Some(selected) = model.selected_piece.take() {
                    if model.board.can_move_piece(&selected, &field) {
                        model.board.move_piece(&selected, &field);

                        let (capture_count, mut fields_to_check) =
                            check_hexes(model, &selected.to_hex());
                        fields_to_check.append(&mut model.board.get_field_edge_neighbors(&field));
                        check_captures(model, &fields_to_check);

                        match model.turn {
                            Color::White => model.white_hexes += capture_count,
                            Color::Black => model.black_hexes += capture_count,
                        }

                        model.last_move = Move::Move(selected, field);
                        model.switch_turns();
                    }
                    clear_selection(model);
                }
            } else if model.exchanging && model.board.is_piece_on_field(&field) {
                model.exchanging = false;
                model.board.remove_piece(&field);
                match model.turn {
                    Color::White => {
                        model.white_hexes -= 2;
                        model.black_pieces -= 1;
                    },
                    Color::Black => {
                        model.black_hexes -= 2;
                        model.white_pieces -= 1;
                    },
                }

                // Players don't collect hexes removed due to an exchange
                let (_, fields_to_check) = check_hexes(model, &field.to_hex());
                check_captures(model, &fields_to_check);

                model.last_move = Move::Exchange(field);
                model.switch_turns();
            } else {
                clear_selection(model);
            },
            Exchange => {
                if model.can_exchange() {
                    model.exchanging = !model.exchanging;
                    clear_selection(model);
                }
            }
            NewGame => *model = Model::new(),
        }
    }
}

fn clear_selection(model: &mut Model) {
    model.selected_piece = None;
    model.available_moves = None;
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
            match model.turn {
                Color::White => model.black_pieces -= 1,
                Color::Black => model.white_pieces -= 1,
            }
        }
    }
}

fn check_hexes(model: &mut Model, coord: &HexCoord) -> (u32, Vec<FieldCoord>) {
    if model.board.is_hex_removable(coord) {
        remove_hex(model, coord)
    } else {
        (0, vec![])
    }
}

fn remove_hex(model: &mut Model, coord: &HexCoord) -> (u32, Vec<FieldCoord>) {
    let mut remove_count = 1;
    model.board.remove_hex(coord);

    let mut fields = vec![];

    for i in 0..6 {
        if let Some(neighbor) = model.board.get_hex_neighbor(coord, i) {
            if model.board.is_hex_removable(&neighbor) {
                let (removed, mut new_fields) = remove_hex(model, &neighbor);
                remove_count += removed;
                fields.append(&mut new_fields);
            } else {
                let field = neighbor.to_field((i + 3) % 6);
                if field.color() != model.turn {
                    fields.push(field);
                }
            }
        }
    }
    (remove_count, fields)
}
