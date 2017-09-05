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

use model::{FieldCoord, Model, Turn};

pub fn update(model: &mut Model, click: Option<FieldCoord>) {
    match click {
        Some(click) => if (model.turn == Turn::White && click.is_white()) ||
            (model.turn == Turn::Black && click.is_black())
        {
            if model.board.is_piece_on_field(&click) {
                model.selected_piece = Some(click);
            } else if let Some(selected) = model.selected_piece.take() {
                if model.board.can_move_piece(&selected, &click) {
                    model.board.move_piece(&selected, &click);
                    model.last_move = Some((Some(selected), click));
                    model.turn.switch_turns();
                }
                model.selected_piece = None;
            }
        } else {
            model.selected_piece = None;
        },
        None => model.selected_piece = None,
    }
}
