/*
 * Copyright (C) 2017 Ryan Huang
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

use model::{Color, Field, FieldCoord, HexCoord, Model};
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
                        check_captures(model, &selected, &field);
                        model.last_move = Some((selected, field));
                        model.switch_turns();
                    }
                    clear_selection(model);
                }
            } else {
                clear_selection(model);
            },
        }
    }
}

fn clear_selection(model: &mut Model) {
    model.selected_piece = None;
    model.available_moves = None;
}

fn check_captures(model: &mut Model, from: &FieldCoord, to: &FieldCoord) {
    let mut fields_to_check = check_hexes(model, &from.to_hex());
    fields_to_check.append(&mut model.board.get_field_edge_neighbors(to));

    for field in fields_to_check {
        if field.color() != model.turn && model.board.is_piece_on_field(&field) &&
            model
                .board
                .get_field_edge_neighbors(&field)
                .into_iter()
                .all(|coord| model.board.get_field(&coord) == &Field::Piece)
        {
            model.board.remove_piece(&field);
            match model.turn {
                Color::White => model.black_pieces -= 1,
                Color::Black => model.white_pieces -= 1,
            }
        }
    }
}

fn check_hexes(model: &mut Model, coord: &HexCoord) -> Vec<FieldCoord> {
    if model.board.is_hex_removable(coord) {
        remove_hex(model, coord)
    } else {
        vec![]
    }
}

fn remove_hex(model: &mut Model, coord: &HexCoord) -> Vec<FieldCoord> {
    model.board.remove_hex(coord);
    match model.turn {
        Color::White => model.white_hexes += 1,
        Color::Black => model.black_hexes += 1,
    }

    let mut fields = vec![];

    for i in 0..6 {
        if let Some(neighbor) = model.board.get_hex_neighbor(coord, i) {
            if model.board.is_hex_removable(&neighbor) {
                fields.append(&mut remove_hex(model, &neighbor));
            } else {
                let field = neighbor.to_field((i + 3) % 6);
                if field.color() != model.turn {
                    fields.push(field);
                }
            }
        }
    }
    fields
}
