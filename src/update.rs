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

use model::{Color, FieldCoord, Model};

pub fn update(model: &mut Model, click: Option<FieldCoord>) {
    match click {
        Some(click) => if (model.turn == Color::White && click.is_white()) ||
            (model.turn == Color::Black && click.is_black())
        {
            if model.board.is_piece_on_field(&click) {
                if model.selected_piece.as_ref() == Some(&click) {
                    clear_selection(model);
                } else {
                    let available_moves = model
                        .board
                        .get_field_vertex_neighbors(&click)
                        .into_iter()
                        .filter(|c| !model.board.is_piece_on_field(c))
                        .collect();

                    model.available_moves = Some(available_moves);
                    model.selected_piece = Some(click);
                }
            } else if let Some(selected) = model.selected_piece.take() {
                if model.board.can_move_piece(&selected, &click) {
                    model.board.move_piece(&selected, &click);
                    model.last_move = Some((Some(selected), click));
                    model.switch_turns();
                }
                clear_selection(model);
            }
        } else {
            clear_selection(model);
        },
        None => clear_selection(model),
    }
}

fn clear_selection(model: &mut Model) {
    model.selected_piece = None;
    model.available_moves = None;
}
