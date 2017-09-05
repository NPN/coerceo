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

mod imgui;

use imgui_sys::{self, ImVec2, ImVec4};

use model::{FieldCoord, HexCoord, Model};
use update::update;
pub use view::imgui::run;

const SQRT_3: f32 = 1.732_050_807_568_877_f32;

// #f3e4cf
const FIELD_WHITE: [f32; 4] = [0.9529, 0.8941, 0.8118, 1.0];
// #998578
const FIELD_BLACK: [f32; 4] = [0.6, 0.5216, 0.4706, 1.0];
// #ffffff
const PIECE_WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
// #000000
const PIECE_OUTLINE: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
// #757575
const PIECE_BLACK: [f32; 4] = [0.4588, 0.4588, 0.4588, 1.0];
// #f7b102
const SELECT_HIGHLIGHT: [f32; 4] = [0.9686, 0.6941, 0.0078, 0.8];
// #ffff00
const LAST_MOVE_HIGHLIGHT: [f32; 4] = [1.0, 1.0, 0.0, 0.8];

pub fn board(model: &mut Model, size: &ImVec2) {
    let mouse_click;
    let mut mouse_pos = ImVec2::default();
    let mut cursor_pos = ImVec2::default();
    unsafe {
        mouse_click = imgui_sys::igIsMouseClicked(0, false);
        imgui_sys::igGetMousePos(&mut mouse_pos);
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

    let board_min = cursor_pos;
    let board_max = add_vec(&cursor_pos, size);
    if mouse_click && board_min.x <= mouse_pos.x && board_min.y <= mouse_pos.y &&
        board_max.x >= mouse_pos.x && board_max.y >= mouse_pos.y
    {
        let click = pixel_to_field(&mouse_pos, &origin, side_len);
        update(model, click);
    }

    if let Some((ref from, ref to)) = model.last_move {
        if let Some(ref from) = *from {
            highlight_field(LAST_MOVE_HIGHLIGHT, from, &origin, side_len);
        }
        highlight_field(LAST_MOVE_HIGHLIGHT, to, &origin, side_len);
    }

    if let Some(ref coord) = model.selected_piece {
        highlight_field(SELECT_HIGHLIGHT, coord, &origin, side_len);
    }

    for hex in model.board.extant_hexes() {
        for f in 0..6 {
            let coord = hex.to_field(f);
            if model.board.is_piece_on_field(&coord) {
                draw_piece(&coord, &origin, side_len);
            }
        }
    }

    unsafe {
        imgui_sys::igDummy(size);
    }
}

macro_rules! im_color {
    ($v:expr) => (imgui_sys::igColorConvertFloat4ToU32(ImVec4::from($v)))
}

fn draw_hex(coord: &HexCoord, origin: &ImVec2, size: f32) {
    for i in 0..6 {
        draw_field(&coord.to_field(i), origin, size);
    }
}

fn draw_field(coord: &FieldCoord, origin: &ImVec2, size: f32) {
    let (v1, v2, v3) = field_vertexes(coord, origin, size);
    unsafe {
        let color = if coord.is_white() {
            im_color!(FIELD_WHITE)
        } else {
            im_color!(FIELD_BLACK)
        };

        let draw_list = imgui_sys::igGetWindowDrawList();
        imgui_sys::ImDrawList_AddTriangleFilled(draw_list, v1, v2, v3, color);
    }
}

fn highlight_field(color: [f32; 4], coord: &FieldCoord, origin: &ImVec2, size: f32) {
    let (v1, v2, v3) = field_vertexes(coord, origin, size);
    unsafe {
        let highlight = im_color!(color);

        let draw_list = imgui_sys::igGetWindowDrawList();
        imgui_sys::ImDrawList_AddTriangleFilled(draw_list, v1, v2, v3, highlight);
    }
}

fn draw_piece(coord: &FieldCoord, origin: &ImVec2, size: f32) {
    let (v1, v2, v3) = field_vertexes(coord, origin, size);
    let center_x = (v1.x + v2.x + v3.x) / 3.0;
    let min_y = if v1.y < v2.y || v1.y < v3.y {
        v1.y
    } else if v2.y < v3.y {
        v2.y
    } else {
        v3.y
    };
    let center_y = if coord.f() % 2 == 0 {
        min_y + size / (2.0 * SQRT_3)
    } else {
        min_y + size / SQRT_3
    };

    let center = ImVec2::new(center_x, center_y);

    const SCALE: f32 = 0.7;

    let v1 = add_vec(&center, &mul_vec(&sub_vec(&v1, &center), SCALE));
    let v2 = add_vec(&center, &mul_vec(&sub_vec(&v2, &center), SCALE));
    let v3 = add_vec(&center, &mul_vec(&sub_vec(&v3, &center), SCALE));

    unsafe {
        let color = if coord.is_white() {
            im_color!(PIECE_WHITE)
        } else {
            im_color!(PIECE_BLACK)
        };

        let draw_list = imgui_sys::igGetWindowDrawList();
        imgui_sys::ImDrawList_AddTriangleFilled(draw_list, v1, v2, v3, color);
        imgui_sys::ImDrawList_AddTriangle(draw_list, v1, v2, v3, im_color!(PIECE_OUTLINE), 2.5);
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

// Algorithm based on http://www.redblobgames.com/grids/hexagons/#pixel-to-hex
fn pixel_to_field(p: &ImVec2, origin: &ImVec2, size: f32) -> Option<FieldCoord> {
    let x = p.x - origin.x;
    let y = p.y - origin.y;

    let q = x * (2.0 / 3.0) / size;
    let r = (-x - SQRT_3 * y) / (size * 3.0);

    if let Some(hex) = round_hex_coord(q, r) {
        /*
           To find the field, we subtract the converted hex coordinates (q, r) from the rounded hex
           coordinates (hex) to get the fractional part of the coordinates. Here is a diagram of a
           single hex, with the fractional coordinates of each of its vertexes in the hex coordinate
           system:

                (-1/3, 2/3) _______ (1/3, 1/3)
                           /\     /\
                          /  \   /  \
                         /    \ /    \
            (-2/3, 1/3) (----(0,0)----) (2/3, -1/3)
                         \    / \    /
                          \  /   \  /
              (-1/3, -1/3) \/_____\/ (1/3, -2/3)

           Suppose our converted coordinates were (q, r) = (1.333, 0.583). Our rounded coordinates
           are hex = (1, 1), and so our fractional coordinates are (0.333, -0.417).

           We now define three linear equations that will split this hexagon into its six fields:

                            _______ y = x (/)
                           /\     /\
                          /  \   /  \
                         /    \ /    \
                        (------X------) y = -x/2 (-)
                         \    / \    /
                          \  /   \  /
                           \/_____\/ y = -2x (\)

           Using these three equations as linear inequalities, we can check any pair of fractional
           coordinates and find which field it is in. (As a reminder, fields are numbered clockwise
           starting from 0 at the top.)

           For our example (0.333, -0.417), we see that:
               y + x/2 >= 0, so our field is above (-), and is either 5, 0, or 1.
               y +  2x >= 0, so our field is to the right of (\), and is either 0 or 1.
               y -   x <  0, so our field is to the right of (/), and is 1.
        */
        let x_diff = q - hex.x() as f32;
        let y_diff = r - hex.y() as f32;
        let mut i = 0;

        if y_diff + x_diff / 2.0 >= 0.0 {
            i |= 0b100;
        }
        if y_diff + x_diff * 2.0 >= 0.0 {
            i |= 0b010;
        }
        if y_diff - x_diff >= 0.0 {
            i |= 0b001;
        }

        // Using a lookup table because nested ifs are too confusing
        const INVALID: u32 = 6;
        let field_lookup = [3, 4, 2, INVALID, INVALID, 5, 1, 0];

        Some(hex.to_field(field_lookup[i]))
    } else {
        None
    }
}

// Algorithm from http://www.redblobgames.com/grids/hexagons/#rounding
fn round_hex_coord(x: f32, y: f32) -> Option<HexCoord> {
    let z = -x - y;

    let mut rx = x.round();
    let mut ry = y.round();
    let rz = z.round();

    let x_diff = (rx - x).abs();
    let y_diff = (ry - y).abs();
    let z_diff = (rz - z).abs();

    if x_diff > y_diff && x_diff > z_diff {
        rx = -ry - rz;
    } else if y_diff > z_diff {
        ry = -rx - rz;
    }

    let rx = rx as i32;
    let ry = ry as i32;

    if HexCoord::is_valid_coord(rx, ry) {
        Some(HexCoord::new(rx, ry))
    } else {
        None
    }
}

fn add_vec(lhs: &ImVec2, rhs: &ImVec2) -> ImVec2 {
    ImVec2::new(lhs.x + rhs.x, lhs.y + rhs.y)
}

fn sub_vec(lhs: &ImVec2, rhs: &ImVec2) -> ImVec2 {
    ImVec2::new(lhs.x - rhs.x, lhs.y - rhs.y)
}

fn mul_vec(v: &ImVec2, c: f32) -> ImVec2 {
    ImVec2::new(v.x * c, v.y * c)
}
