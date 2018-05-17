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

use imgui_sys::{self, ImVec2};

use model::{FieldCoord, Model, Move};
use vec2::Vec2;
use view::board_parts::*;
use view::Event;

const SQRT_3: f32 = 1.732_050_807_568_877_f32;

// #f7b102
const SELECT_HIGHLIGHT: [f32; 4] = [0.9686, 0.6941, 0.0078, 0.8];
// #ffff00
const LAST_MOVE_HIGHLIGHT: [f32; 4] = [1.0, 1.0, 0.0, 0.8];
// #ff0000
const EXCHANGE_HIGHLIGHT: [f32; 4] = [1.0, 0.0, 0.0, 0.8];

pub fn board(model: &Model, size: Vec2) -> Option<Event> {
    let mouse_click;
    let mut mouse_pos = ImVec2::default();
    let mut cursor_pos = ImVec2::default();
    unsafe {
        mouse_click = imgui_sys::igIsMouseClicked(0, false);
        imgui_sys::igGetMousePos(&mut mouse_pos);
        imgui_sys::igGetCursorScreenPos(&mut cursor_pos);
    }
    let cursor_pos = Vec2::from(cursor_pos);
    let mouse_pos = Vec2::from(mouse_pos);

    let side_len = (size.x / 8.0).min(size.y / (5.0 * SQRT_3));
    let origin = cursor_pos + size / 2.0;

    let extant_hexes = model.board.extant_hexes();

    for hex in &extant_hexes {
        draw_hex(hex, origin, side_len);
    }

    if let Some(mv) = model.last_move {
        match mv {
            Move::Exchange(exchanged, color) => {
                if model
                    .board
                    .is_hex_extant(exchanged.trailing_zeros() as usize / 3)
                {
                    let exchanged = FieldCoord::from_bitboard(exchanged, color);
                    highlight_field(EXCHANGE_HIGHLIGHT, &exchanged, origin, side_len);
                }
            }
            Move::Move(from, to, color) => {
                if model
                    .board
                    .is_hex_extant(from.trailing_zeros() as usize / 3)
                {
                    let from = FieldCoord::from_bitboard(from, color);
                    highlight_field(LAST_MOVE_HIGHLIGHT, &from, origin, side_len);
                }
                let to = FieldCoord::from_bitboard(to, color);
                highlight_field(LAST_MOVE_HIGHLIGHT, &to, origin, side_len);
            }
        }
    }

    if let Some(ref coord) = model.selected_piece {
        highlight_field(SELECT_HIGHLIGHT, coord, origin, side_len);
        for coord in model.board.available_moves_for_piece(coord) {
            highlight_field_dot(SELECT_HIGHLIGHT, &coord, origin, side_len);
        }
    }

    let mut hover_field = pixel_to_field(mouse_pos, origin, side_len);
    if !hover_field
        .map(|field| model.board.is_hex_extant(field.to_hex().to_index()))
        .unwrap_or(false)
    {
        hover_field = None;
    }
    if let Some(ref coord) = hover_field {
        if model.exchanging && coord.color() != model.board.turn()
            && model.board.is_piece_on_field(coord)
        {
            highlight_field(EXCHANGE_HIGHLIGHT, coord, origin, side_len);
        }
    }

    for hex in &extant_hexes {
        for f in 0..6 {
            let coord = hex.to_field(f);
            if model.board.is_piece_on_field(&coord) {
                draw_piece(&coord, origin, side_len);
            }
        }
    }

    unsafe {
        imgui_sys::igDummy(&size.into());
    }

    if mouse_click {
        hover_field.map(Event::Click)
    } else {
        None
    }
}
