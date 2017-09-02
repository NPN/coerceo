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

extern crate glium;
#[macro_use]
extern crate imgui;
extern crate imgui_glium_renderer;
extern crate imgui_sys;

mod model;
mod view;

use imgui::Ui;
use imgui_sys::ImVec2;

use model::Model;

fn main() {
    let mut model = Model::new();

    view::run(
        String::from("Coerceo"),
        (800, 800),
        [1.0, 1.0, 1.0, 1.0],
        |ui, size| test_ui(ui, size, &mut model),
    );
}

fn test_ui(ui: &Ui, size: (f32, f32), model: &mut Model) -> bool {
    unsafe {
        imgui_sys::igPushStyleVar(imgui_sys::ImGuiStyleVar::WindowRounding, 0.0);
    }

    ui.window(im_str!("Coerceo"))
        .size(size, imgui::ImGuiSetCond_Always)
        .position((0.0, 0.0), imgui::ImGuiSetCond_Once)
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .collapsible(false)
        .build(|| {
            ui.text(im_str!("Welcome to Coerceo!"));

            view::board::board(model, &ImVec2::new(600.0, 600.0));

            ui.text(im_str!("Look at the board!"));
        });

    unsafe {
        imgui_sys::igPopStyleVar(1);
    }

    true
}
