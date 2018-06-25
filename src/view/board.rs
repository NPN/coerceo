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

use imgui::{ImMouseButton, Ui};

use model::bitboard::BitBoardExt;
use model::{FieldCoord, Model, Move};
use view::board_parts::*;
use view::vec2::Vec2;
use view::Event;

const SQRT_3: f32 = 1.732_050_807_568_877_f32;

// Color format is 0xaa_bb_gg_rr
const SELECT_HIGHLIGHT: u32 = 0xcc_35_bf_ff;
const LAST_MOVE_HIGHLIGHT: u32 = 0xc3_49_f8_f2;
/// The highlight for a piece capture by surrounding or exchanging.
const CAPTURE_HIGHLIGHT: u32 = 0xcf_40_40_ff;

/// The alpha used for a removed hex and any highlights on it.
const REMOVED_HEX_ALPHA: u8 = 0x50;
const EXTANT_HEX_ALPHA: u8 = 0xff;

pub fn board(ui: &Ui, model: &Model, size: Vec2) -> Option<Event> {
    let mouse_click = ui.imgui().is_mouse_clicked(ImMouseButton::Left);
    let mouse_pos = Vec2::from(ui.imgui().mouse_pos());
    let cursor_pos = Vec2::from(ui.get_cursor_screen_pos());

    let side_len = {
        // hex_spacing  =          m * side_len + b
        // board_width  =          8 * side_len + 6 * SQRT_3 * hex_spacing
        // board_height = 5 * SQRT_3 * side_len +          4 * hex_spacing
        let (m, b) = HEX_SPACING_COEFF;
        let width = (size.x - 6.0 * SQRT_3 * b) / (8.0 + 6.0 * SQRT_3 * m);
        let height = (size.y - 4.0 * b) / (5.0 * SQRT_3 + 4.0 * m);

        width.min(height)
    };
    let origin = cursor_pos + size / 2.0;

    let extant_hexes = model.board.extant_hexes();

    for hex in &extant_hexes {
        draw_hex(ui, EXTANT_HEX_ALPHA, hex, origin, side_len);
    }

    if let Some(ref mv) = model.last_move {
        for hex in &mv.removed_hexes {
            draw_hex(ui, REMOVED_HEX_ALPHA, &hex, origin, side_len);
        }

        for piece in &mv.removed_pieces {
            let color = if model.board.is_hex_extant(piece.to_hex().to_index()) {
                CAPTURE_HIGHLIGHT
            } else {
                set_alpha(CAPTURE_HIGHLIGHT, REMOVED_HEX_ALPHA)
            };
            draw_field(ui, color, piece, origin, side_len);
        }

        if let Move::Move(from, to, color) = mv.mv {
            let from_color = if model.board.is_hex_extant(from.to_index()) {
                LAST_MOVE_HIGHLIGHT
            } else {
                set_alpha(LAST_MOVE_HIGHLIGHT, REMOVED_HEX_ALPHA)
            };

            let from = FieldCoord::from_bitboard(from, color);
            draw_field(ui, from_color, &from, origin, side_len);

            let to = FieldCoord::from_bitboard(to, color);
            draw_field(ui, LAST_MOVE_HIGHLIGHT, &to, origin, side_len);
        }
    }

    if let Some(ref coord) = model.selected_piece {
        draw_field(ui, SELECT_HIGHLIGHT, coord, origin, side_len);
        for coord in model.board.available_moves_for_piece(coord) {
            draw_field_dot(ui, SELECT_HIGHLIGHT, &coord, origin, side_len);
        }
    }

    let hover_field = pixel_to_field(mouse_pos, origin, side_len)
        .filter(|field| model.board.is_hex_extant(field.to_hex().to_index()));

    if let Some(ref coord) = hover_field {
        if model.exchanging
            && coord.color() != model.board.turn
            && model.board.is_piece_on_field(coord)
        {
            draw_field(ui, CAPTURE_HIGHLIGHT, coord, origin, side_len);
        }
    }

    for hex in &extant_hexes {
        for f in 0..6 {
            let coord = hex.to_field(f);
            if model.board.is_piece_on_field(&coord) {
                draw_piece(ui, &coord, origin, side_len);
            }
        }
    }

    ui.dummy(size);

    hover_field.filter(|_| mouse_click).map(Event::Click)
}
