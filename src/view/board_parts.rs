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

use imgui_sys::{self, ImVec4};

use model::{Color, FieldCoord, HexCoord};
use vec2::Vec2;

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

macro_rules! im_color {
    ($v:expr) => {
        imgui_sys::igColorConvertFloat4ToU32(ImVec4::from($v))
    };
}

pub fn draw_hex(coord: &HexCoord, origin: Vec2, size: f32) {
    for i in 0..6 {
        draw_field(&coord.to_field(i), origin, size);
    }
}

fn draw_field(coord: &FieldCoord, origin: Vec2, size: f32) {
    let (v1, v2, v3) = field_vertexes(coord, origin, size);
    unsafe {
        let color = match coord.color() {
            Color::White => im_color!(FIELD_WHITE),
            Color::Black => im_color!(FIELD_BLACK),
        };

        let draw_list = imgui_sys::igGetWindowDrawList();
        imgui_sys::ImDrawList_AddTriangleFilled(draw_list, v1.into(), v2.into(), v3.into(), color);
    }
}

pub fn highlight_field(color: [f32; 4], coord: &FieldCoord, origin: Vec2, size: f32) {
    let (v1, v2, v3) = field_vertexes(coord, origin, size);
    unsafe {
        let highlight = im_color!(color);

        let draw_list = imgui_sys::igGetWindowDrawList();
        imgui_sys::ImDrawList_AddTriangleFilled(
            draw_list,
            v1.into(),
            v2.into(),
            v3.into(),
            highlight,
        );
    }
}

pub fn highlight_field_dot(color: [f32; 4], coord: &FieldCoord, origin: Vec2, size: f32) {
    let center = field_center(coord, origin, size);
    unsafe {
        let highlight = im_color!(color);

        let draw_list = imgui_sys::igGetWindowDrawList();
        imgui_sys::ImDrawList_AddCircleFilled(
            draw_list,
            center.into(),
            size / (4.0 * SQRT_3),
            highlight,
            15,
        );
    }
}

pub fn draw_piece(coord: &FieldCoord, origin: Vec2, size: f32) {
    let (v1, v2, v3) = field_vertexes(coord, origin, size);
    let center = field_center(coord, origin, size);

    const SCALE: f32 = 0.7;

    let v1 = center + (v1 - center) * SCALE;
    let v2 = center + (v2 - center) * SCALE;
    let v3 = center + (v3 - center) * SCALE;

    // Linear equation derived by human testing and regression
    let outline_size = 0.032 * size - 0.335;

    unsafe {
        let color = match coord.color() {
            Color::White => im_color!(PIECE_WHITE),
            Color::Black => im_color!(PIECE_BLACK),
        };

        let draw_list = imgui_sys::igGetWindowDrawList();
        imgui_sys::ImDrawList_AddTriangleFilled(draw_list, v1.into(), v2.into(), v3.into(), color);
        imgui_sys::ImDrawList_AddTriangle(
            draw_list,
            v1.into(),
            v2.into(),
            v3.into(),
            im_color!(PIECE_OUTLINE),
            outline_size,
        );
    }
}

fn field_center(coord: &FieldCoord, origin: Vec2, size: f32) -> Vec2 {
    let (v1, v2, v3) = field_vertexes(coord, origin, size);
    let center_x = (v1.x + v2.x + v3.x) / 3.0;
    let min_y = (v1.y).min(v2.y).min(v3.y);
    let center_y = match coord.color() {
        Color::White => min_y + size / SQRT_3,
        Color::Black => min_y + size / (2.0 * SQRT_3),
    };

    Vec2::new(center_x, center_y)
}

fn field_vertexes(coord: &FieldCoord, origin: Vec2, size: f32) -> (Vec2, Vec2, Vec2) {
    let center = hex_to_pixel(&coord.to_hex(), origin, size);
    let height = size * SQRT_3 / 2.0;

    let west = Vec2::new(-size, 0.0);
    let east = Vec2::new(size, 0.0);

    let northwest = Vec2::new(-size / 2.0, -height);
    let southwest = Vec2::new(-size / 2.0, height);
    let northeast = Vec2::new(size / 2.0, -height);
    let southeast = Vec2::new(size / 2.0, height);

    let v1;
    let v2;
    match coord.f() {
        0 => {
            v1 = center + northwest;
            v2 = center + northeast;
        }
        1 => {
            v1 = center + northeast;
            v2 = center + east;
        }
        2 => {
            v1 = center + east;
            v2 = center + southeast;
        }
        3 => {
            v1 = center + southeast;
            v2 = center + southwest;
        }
        4 => {
            v1 = center + southwest;
            v2 = center + west;
        }
        5 => {
            v1 = center + west;
            v2 = center + northwest;
        }
        _ => unreachable!(),
    };

    (center, v1, v2)
}

// Algorithm based on http://www.redblobgames.com/grids/hexagons/#hex-to-pixel
fn hex_to_pixel(coord: &HexCoord, origin: Vec2, size: f32) -> Vec2 {
    let x = f32::from(coord.x());
    let y = f32::from(coord.y());

    let p = Vec2::new(size * (3.0 / 2.0) * x, size * -SQRT_3 * (x / 2.0 + y));

    origin + p
}

// Algorithm based on http://www.redblobgames.com/grids/hexagons/#pixel-to-hex
pub fn pixel_to_field(p: Vec2, origin: Vec2, size: f32) -> Option<FieldCoord> {
    let v = p - origin;

    let q = v.x * (2.0 / 3.0) / size;
    let r = (-v.x - SQRT_3 * v.y) / (size * 3.0);

    round_hex_coord(q, r).and_then(|hex| {
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
        let x_diff = q - f32::from(hex.x());
        let y_diff = r - f32::from(hex.y());
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

        Some(hex.to_field(match i {
            0 => 3,
            1 => 4,
            2 => 2,
            5 => 5,
            6 => 1,
            7 => 0,
            _ => unreachable!(),
        }))
    })
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

    HexCoord::try_new(rx as i8, ry as i8)
}
