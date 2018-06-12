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

use imgui_sys;

use model::{Color, ColorMap, FieldCoord, HexCoord};
use view::vec2::Vec2;

const SQRT_3: f32 = 1.732_050_807_568_877_f32;

// Slope and y-intercept
pub const HEX_SPACING_COEFF: (f32, f32) = (0.0331, 1.45);

// Color format is 0xaa_bb_gg_rr
const FIELD_COLORS: ColorMap<u32> = ColorMap {
    white: 0xff_e9_ef_f3,
    black: 0xff_78_99_83,
};
const PIECE_OUTLINE: u32 = 0xff_23_23_23;
const PIECE_COLORS: ColorMap<[u32; 3]> = ColorMap {
    white: [
        // Light, medium, and dark colors
        0xff_f8_f8_f8,
        0xff_e0_e0_e0,
        0xff_bd_bd_bd,
    ],
    black: [
        // Medium, light, and dark colors
        0xff_68_68_68,
        0xff_88_88_88,
        0xff_58_58_58,
    ],
};

pub fn draw_hex(coord: &HexCoord, origin: Vec2, size: f32) {
    for i in 0..6 {
        draw_field(&coord.to_field(i), origin, size);
    }
}

fn draw_field(coord: &FieldCoord, origin: Vec2, size: f32) {
    let (v1, v2, v3) = field_vertexes(coord, origin, size);
    unsafe {
        let draw_list = imgui_sys::igGetWindowDrawList();
        imgui_sys::ImDrawList_AddTriangleFilled(
            draw_list,
            v1.into(),
            v2.into(),
            v3.into(),
            FIELD_COLORS.get(coord.color()),
        );
    }
}

pub fn highlight_field(color: u32, coord: &FieldCoord, origin: Vec2, size: f32) {
    let (v1, v2, v3) = field_vertexes(coord, origin, size);
    unsafe {
        let draw_list = imgui_sys::igGetWindowDrawList();
        imgui_sys::ImDrawList_AddTriangleFilled(draw_list, v1.into(), v2.into(), v3.into(), color);
    }
}

pub fn highlight_field_dot(color: u32, coord: &FieldCoord, origin: Vec2, size: f32) {
    let center = field_center(coord, origin, size);
    unsafe {
        let draw_list = imgui_sys::igGetWindowDrawList();
        imgui_sys::ImDrawList_AddCircleFilled(
            draw_list,
            center.into(),
            size / (4.0 * SQRT_3),
            color,
            15,
        );
    }
}

pub fn draw_piece(coord: &FieldCoord, origin: Vec2, size: f32) {
    let (v1, v2, v3) = field_vertexes(coord, origin, size);
    let center = field_center(coord, origin, size);

    const SCALE: f32 = 0.75;

    let v1 = (center + (v1 - center) * SCALE).into();
    let v2 = (center + (v2 - center) * SCALE).into();
    let v3 = (center + (v3 - center) * SCALE).into();
    let center = center.into();

    // Linear equation derived by human testing and regression
    let outline_size = 0.032 * size - 0.535;

    unsafe {
        let draw_list = imgui_sys::igGetWindowDrawList();

        let colors = PIECE_COLORS.get_ref(coord.color());
        imgui_sys::ImDrawList_AddTriangleFilled(draw_list, v1, v2, center, colors[0]);
        imgui_sys::ImDrawList_AddTriangleFilled(draw_list, v2, v3, center, colors[1]);
        imgui_sys::ImDrawList_AddTriangleFilled(draw_list, v3, v1, center, colors[2]);

        imgui_sys::ImDrawList_AddTriangle(draw_list, v1, v2, v3, PIECE_OUTLINE, outline_size);
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

    let west = center + Vec2::new(-size, 0.0);
    let east = center + Vec2::new(size, 0.0);

    let northwest = center + Vec2::new(-size / 2.0, -height);
    let southwest = center + Vec2::new(-size / 2.0, height);
    let northeast = center + Vec2::new(size / 2.0, -height);
    let southeast = center + Vec2::new(size / 2.0, height);

    // Vertexes are ordered clockwise for draw_piece to shade the sides.
    match coord.f() {
        0 => (center, northwest, northeast),
        1 => (center, northeast, east),
        2 => (southeast, center, east),
        3 => (southwest, center, southeast),
        4 => (southwest, west, center),
        5 => (west, northwest, center),
        _ => unreachable!(),
    }
}

fn hex_spacing(size: f32) -> f32 {
    // Again, equation derived from human judgment and linear regression
    HEX_SPACING_COEFF.0 * size + HEX_SPACING_COEFF.1
}

// Algorithm based on http://www.redblobgames.com/grids/hexagons/#hex-to-pixel
fn hex_to_pixel(coord: &HexCoord, origin: Vec2, size: f32) -> Vec2 {
    let x = f32::from(coord.x());
    let y = f32::from(coord.y());

    let p = Vec2::new(size * (3.0 / 2.0) * x, size * -SQRT_3 * (x / 2.0 + y));

    origin + p * (1.0 + hex_spacing(size) / (size * SQRT_3))
}

// Algorithm based on http://www.redblobgames.com/grids/hexagons/#pixel-to-hex
pub fn pixel_to_field(p: Vec2, origin: Vec2, size: f32) -> Option<FieldCoord> {
    // Finding the hex is tricky because the hexes have gaps between them.
    // First, we find the rounded hex coordinate with a scaled up size that accounts for the gap.

    let larger_size = size + hex_spacing(size) / SQRT_3;
    round_hex_coord(p - origin, larger_size)
        .and_then(|hex| {
            // This gives us the correct hex--if each hex was big enough to close the gaps. But since
            // there are gaps, we need to exclude pixels which lie in those gaps. We do this by finding
            // the rounded hex coordinate again with the correct size. If the pixel is still in the right
            // hex, then we know we are not in a gap.

            let pixel_offset = p - hex_to_pixel(&hex, origin, size);
            if HexCoord::try_new(0, 0) == round_hex_coord(pixel_offset, size) {
                Some((hex, pixel_to_hex_uniform(pixel_offset, size)))
            } else {
                None
            }
        })
        .and_then(|(hex, frac_hex)| {
            /*
               To find the field, we start with the fractional coordinates. Here is a diagram of a single
               hex, with the fractional coordinates of each of its vertexes in the hex coordinate system:

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
            let Vec2 { x, y } = frac_hex;
            let mut i = 0;

            if y + x / 2.0 >= 0.0 {
                i |= 0b100;
            }
            if y + x * 2.0 >= 0.0 {
                i |= 0b010;
            }
            if y - x >= 0.0 {
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
fn round_hex_coord(v: Vec2, size: f32) -> Option<HexCoord> {
    let Vec2 { x, y } = pixel_to_hex_uniform(v, size);
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

// This function does not take into account gaps. It is only a helper for the other functions.
fn pixel_to_hex_uniform(v: Vec2, size: f32) -> Vec2 {
    Vec2 {
        x: v.x * (2.0 / 3.0) / size,
        y: (-v.x - SQRT_3 * v.y) / (size * 3.0),
    }
}
