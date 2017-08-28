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

mod model;
mod view;

use imgui::Ui;

fn main() {
    view::run(
        String::from("Coerceo"),
        (800, 800),
        [1.0, 1.0, 1.0, 1.0],
        test_ui,
    );
}

fn test_ui(ui: &Ui) -> bool {
    ui.window(im_str!("Coerceo"))
        .size((600.0, 600.0), imgui::ImGuiSetCond_FirstUseEver)
        .build(|| { ui.text(im_str!("Welcome to Coerceo!")); });

    true
}
