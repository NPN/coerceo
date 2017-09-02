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

use imgui_sys::{self, ImVec2, ImVec4};

use model::{FieldCoord, HexCoord, Model};

const SQRT_3: f32 = 1.732_050_807_568_877_f32;

pub fn board(model: &mut Model, size: &ImVec2) {
    let mut cursor_pos = ImVec2::default();
    unsafe {
        imgui_sys::igGetCursorScreenPos(&mut cursor_pos);
    }

    // We want to fit the board into the size vector given us. Since the board is slightly taller
    // than it is wide, we take the height as our constraining dimension, and calculate the side
    // length of a triangle on the board from it.
    let side_len = size.y / (5.0 * SQRT_3);
    let origin = ImVec2::new(cursor_pos.x + size.x / 2.0, cursor_pos.y + size.y / 2.0);
    for hex in model.board.extant_hexes() {
        draw_hex(&hex, &origin, side_len);
    }

    unsafe {
        imgui_sys::igDummy(size);
    }
}

fn draw_hex(coord: &HexCoord, origin: &ImVec2, size: f32) {
    for i in 0..6 {
        draw_field(&coord.to_field(i), origin, size);
    }
}

fn draw_field(coord: &FieldCoord, origin: &ImVec2, size: f32) {
    let (v1, v2, v3) = field_vertexes(coord, origin, size);
    unsafe {
        let white = imgui_sys::igColorConvertFloat4ToU32(ImVec4::new(1.0, 1.0, 1.0, 1.0));
        let black = imgui_sys::igColorConvertFloat4ToU32(ImVec4::new(0.0, 0.0, 0.0, 1.0));

        let color = if coord.f() % 2 == 0 { white } else { black };

        let draw_list = imgui_sys::igGetWindowDrawList();
        imgui_sys::ImDrawList_AddTriangleFilled(draw_list, v1, v2, v3, color);
    }
}

fn field_vertexes(coord: &FieldCoord, origin: &ImVec2, size: f32) -> (ImVec2, ImVec2, ImVec2) {
    let center = hex_to_pixel(&coord.to_hex(), origin, size);
    let height = size * SQRT_3 / 2.0;

    let v1;
    let v2;
    match coord.f() {
        0 => {
            v1 = ImVec2::new(center.x - size / 2.0, center.y - height);
            v2 = ImVec2::new(center.x + size / 2.0, center.y - height);
        }
        1 => {
            v1 = ImVec2::new(center.x + size / 2.0, center.y - height);
            v2 = ImVec2::new(center.x + size, center.y);
        }
        2 => {
            v1 = ImVec2::new(center.x + size, center.y);
            v2 = ImVec2::new(center.x + size / 2.0, center.y + height);
        }
        3 => {
            v1 = ImVec2::new(center.x + size / 2.0, center.y + height);
            v2 = ImVec2::new(center.x - size / 2.0, center.y + height);
        }
        4 => {
            v1 = ImVec2::new(center.x - size / 2.0, center.y + height);
            v2 = ImVec2::new(center.x - size, center.y);
        }
        5 => {
            v1 = ImVec2::new(center.x - size, center.y);
            v2 = ImVec2::new(center.x - size / 2.0, center.y - height);
        }
        _ => panic!("You made the impossible possible."),
    };

    (center, v1, v2)
}

// Algorithm based on http://www.redblobgames.com/grids/hexagons/#hex-to-pixel
fn hex_to_pixel(coord: &HexCoord, origin: &ImVec2, size: f32) -> ImVec2 {
    let x = coord.x() as f32;
    let y = coord.y() as f32;

    let px = size * (3.0 / 2.0) * x;
    let py = size * -SQRT_3 * (x / 2.0 + y);

    ImVec2::new(px + origin.x, py + origin.y)
}
