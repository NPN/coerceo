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

mod board;
mod sys;

use imgui::{self, ImVec2, Ui};
use imgui_sys;

use model::{Color, FieldCoord, Model};
use self::board::board;
pub use self::sys::run;

pub enum Event {
    Click(FieldCoord),
}

pub fn draw(ui: &Ui, size: (f32, f32), model: &Model) -> Option<Event> {
    unsafe {
        imgui_sys::igPushStyleVar(imgui_sys::ImGuiStyleVar::WindowRounding, 0.0);
    }

    let mut event = None;
    ui.window(im_str!("Coerceo"))
        .size(size, imgui::ImGuiSetCond_Always)
        .position((0.0, 0.0), imgui::ImGuiSetCond_Once)
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .build(|| {
            ui.text(im_str!("Welcome to Coerceo!"));

            event = board(model, &ImVec2::new(600.0, 600.0));

            match model.turn {
                Color::White => ui.text(im_str!("It's white's turn.")),
                Color::Black => ui.text(im_str!("It's black's turn.")),
            }
        });

    unsafe {
        imgui_sys::igPopStyleVar(1);
    }

    event
}
