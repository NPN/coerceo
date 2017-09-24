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

use model::Model;
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
                        model.last_move = Some((Some(selected), field));
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
