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

use imgui::{ImGuiCond, ImVec2, Ui};
use imgui_sys;

use model::{Color, FieldCoord, Model};
use self::board::board;
pub use self::sys::run;

pub enum Event {
    Click(FieldCoord),
    NewGame,
}

pub fn draw(ui: &Ui, size: (f32, f32), model: &Model) -> Option<Event> {
    unsafe {
        imgui_sys::igPushStyleVar(imgui_sys::ImGuiStyleVar::WindowRounding, 0.0);
    }

    let mut event = None;

    ui.main_menu_bar(|| {
        ui.menu(im_str!("Game")).build(|| {
            if ui.menu_item(im_str!("New game")).build() {
                event = Some(Event::NewGame);
            }
        });
    });

    ui.window(im_str!("Coerceo"))
        .size(size, ImGuiCond::Always)
        .position((0.0, 19.0), ImGuiCond::Once)
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .build(|| {
            ui.text("Welcome to Coerceo!");

            if event.is_none() {
                event = board(model, &ImVec2::new(size.0 - 16.0, size.1 - 100.0));
            } else {
                board(model, &ImVec2::new(size.0 - 16.0, size.1 - 100.0));
            }

            if model.white_pieces == 0 {
                ui.text("Black wins!");
            } else if model.black_pieces == 0 {
                ui.text("White wins!");
            } else {
                match model.turn {
                    Color::White => ui.text("It's white's turn."),
                    Color::Black => ui.text("It's black's turn."),
                }

                ui.text(format!(
                    "White has {} piece(s) left and {} captured hex(es).",
                    model.white_pieces, model.white_hexes,
                ));
                ui.text(format!(
                    "Black has {} piece(s) left and {} captured hex(es).",
                    model.black_pieces, model.black_hexes,
                ));
            }
        });

    unsafe {
        imgui_sys::igPopStyleVar(1);
    }

    event
}
