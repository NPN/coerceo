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

use imgui::{ImGuiCond, ImStr, ImVec2, Ui};
use imgui_sys;

use model::{Color, ColorMap, FieldCoord, Model, PlayerType};
use vec2::Vec2;
use self::board::board;
pub use self::sys::run;

#[derive(PartialEq)]
pub enum Event {
    Click(FieldCoord),
    Exchange,
    NewGame(ColorMap<PlayerType>),
    Resign,
    Undo,
    Redo,
}

pub fn draw(ui: &Ui, size: (f32, f32), model: &Model) -> Option<Event> {
    unsafe {
        imgui_sys::igPushStyleVar(imgui_sys::ImGuiStyleVar::WindowRounding, 0.0);
    }

    let mut event = None;

    ui.main_menu_bar(|| {
        ui.menu(im_str!("Game")).build(|| {
            ui.menu_item(im_str!("New game")).enabled(false).build();
            if ui.menu_item(im_str!("Human vs. Human")).build() {
                insert_if_empty(
                    &mut event,
                    Event::NewGame(ColorMap::new(PlayerType::Human, PlayerType::Human)),
                );
            }
            if ui.menu_item(im_str!("Human vs. Computer")).build() {
                insert_if_empty(
                    &mut event,
                    Event::NewGame(ColorMap::new(PlayerType::Human, PlayerType::Computer)),
                );
            }
            if ui.menu_item(im_str!("Computer vs. Human")).build() {
                insert_if_empty(
                    &mut event,
                    Event::NewGame(ColorMap::new(PlayerType::Computer, PlayerType::Human)),
                );
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
            ui.text(format!(
                "{:?} vs. {:?}",
                model.players.white, model.players.black
            ));

            let click = board(model, Vec2::new(size.0 - 16.0, size.1 - 170.0));

            use model::GameResult::*;
            match model.game_result {
                Win(color) => {
                    ui.text(format!("{:?} wins!", color));
                    if model.can_undo() && ui.button(im_str!("Undo"), Vec2::new(100.0, 20.0)) {
                        insert_if_empty(&mut event, Event::Undo);
                    }
                }
                InProgress => {
                    if let Some(click) = click {
                        insert_if_empty(&mut event, click);
                    }

                    ui.text(format!("It's {:?}'s turn.", model.board.turn()));

                    ui.text(format!(
                        "{:?} has {} piece(s) left and {} captured hex(es).",
                        Color::White,
                        model.board.pieces(Color::White),
                        model.board.hexes(Color::White),
                    ));
                    ui.text(format!(
                        "{:?} has {} piece(s) left and {} captured hex(es).",
                        Color::Black,
                        model.board.pieces(Color::Black),
                        model.board.hexes(Color::Black),
                    ));

                    let button_size = Vec2::new(120.0, 20.0);
                    horz_button_layout(
                        ui,
                        vec![
                            (true, im_str!("Resign"), Event::Resign),
                            (
                                model.board.can_exchange() && !model.is_ai_turn(),
                                if model.exchanging {
                                    im_str!("Stop Exchanging")
                                } else {
                                    im_str!("Exchange")
                                },
                                Event::Exchange,
                            ),
                        ],
                        &button_size,
                        &mut event,
                    );
                    horz_button_layout(
                        ui,
                        vec![
                            (model.can_undo(), im_str!("Undo"), Event::Undo),
                            (model.can_redo(), im_str!("Redo"), Event::Redo),
                        ],
                        &button_size,
                        &mut event,
                    );
                }
            }
        });

    unsafe {
        imgui_sys::igPopStyleVar(1);
    }

    event
}

fn horz_button_layout(
    ui: &Ui,
    buttons: Vec<(bool, &ImStr, Event)>,
    size: &Vec2,
    event: &mut Option<Event>,
) {
    let size: ImVec2 = (*size).into();

    for (show, label, action) in buttons {
        if show {
            if ui.button(label, size) {
                insert_if_empty(event, action);
            }
        } else {
            unsafe {
                imgui_sys::igDummy(&size);
            }
        }
        ui.same_line(0.0);
    }
    ui.new_line();
}

fn insert_if_empty<T>(a: &mut Option<T>, b: T) {
    if a.is_none() {
        *a = Some(b);
    }
}
