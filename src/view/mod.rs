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

mod board;
mod board_parts;
mod sys;

use imgui::{ImGuiCond, Ui};
use imgui_sys;

use model::{Color, FieldCoord, Model};
use vec2::Vec2;
use self::board::board;
pub use self::sys::run;

#[derive(PartialEq)]
pub enum Event {
    Click(FieldCoord),
    Exchange,
    NewGame,
    Resign,
}

pub fn draw(ui: &Ui, size: (f32, f32), model: &Model) -> Option<Event> {
    unsafe {
        imgui_sys::igPushStyleVar(imgui_sys::ImGuiStyleVar::WindowRounding, 0.0);
    }

    let mut event = None;

    ui.main_menu_bar(|| {
        ui.menu(im_str!("Game")).build(|| {
            if ui.menu_item(im_str!("New game")).build() {
                insert_if_empty(&mut event, Event::NewGame);
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

            if let Some(click) = board(model, Vec2::new(size.0 - 16.0, size.1 - 135.0)) {
                insert_if_empty(&mut event, click);
            }

            use model::GameResult::*;
            match model.game_result {
                BlackWin => ui.text("Black wins!"),
                WhiteWin => ui.text("White wins!"),
                InProgress => {
                    match model.turn {
                        Color::White => ui.text("It's white's turn."),
                        Color::Black => ui.text("It's black's turn."),
                    }

                    ui.text(format!(
                        "White has {} piece(s) left and {} captured hex(es).",
                        model.board.white_pieces(),
                        model.white_hexes,
                    ));
                    ui.text(format!(
                        "Black has {} piece(s) left and {} captured hex(es).",
                        model.board.black_pieces(),
                        model.black_hexes,
                    ));

                    if ui.button(im_str!("Resign"), Vec2::new(100.0, 20.0)) {
                        insert_if_empty(&mut event, Event::Resign);
                    }
                    ui.same_line(0.0);
                    if model.can_exchange() {
                        let label = if model.exchanging {
                            im_str!("Stop Exchanging")
                        } else {
                            im_str!("Exchange")
                        };
                        if ui.button(label, Vec2::new(120.0, 20.0)) {
                            insert_if_empty(&mut event, Event::Exchange);
                        }
                    }
                }
            }
        });

    unsafe {
        imgui_sys::igPopStyleVar(1);
    }

    event
}

fn insert_if_empty<T>(a: &mut Option<T>, b: T) {
    if a.is_none() {
        *a = Some(b);
    }
}
