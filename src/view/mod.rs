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
mod vec2;

use imgui::{ImGuiCond, ImStr, ImVec2, Ui};
use imgui_sys;

use self::board::board;
pub use self::sys::run;
use self::vec2::Vec2;
use model::{Color, ColorMap, Model, Player};
use update::Event;

pub fn draw(ui: &Ui, size: (f32, f32), model: &Model) -> Option<Event> {
    unsafe {
        imgui_sys::igPushStyleVar(imgui_sys::ImGuiStyleVar::WindowRounding, 0.0);
    }

    let mut event = None;

    ui.main_menu_bar(|| {
        ui.menu(im_str!("Game")).build(|| {
            ui.menu_item(im_str!("New game")).enabled(false).build();

            use self::Player::*;
            if ui.menu_item(im_str!("Human vs. Human")).build() {
                insert_if_empty(&mut event, Event::NewGame(ColorMap::new(Human, Human)));
            }
            if ui.menu_item(im_str!("Human vs. Computer")).build() {
                insert_if_empty(&mut event, Event::NewGame(ColorMap::new(Human, Computer)));
            }
            if ui.menu_item(im_str!("Computer vs. Human")).build() {
                insert_if_empty(&mut event, Event::NewGame(ColorMap::new(Computer, Human)));
            }
            if ui.menu_item(im_str!("Computer vs. Computer")).build() {
                insert_if_empty(
                    &mut event,
                    Event::NewGame(ColorMap::new(Computer, Computer)),
                );
            }
            ui.separator();
            if ui.menu_item(im_str!("Quit")).build() {
                insert_if_empty(&mut event, Event::Quit);
            }
        });

        let mut depth = model.ai_search_depth as i32;
        ui.menu(im_str!("Computer")).build(|| {
            ui.slider_int(im_str!("Search depth"), &mut depth, 1, 7)
                .build();
            if ui.is_item_hovered() {
                ui.tooltip_text("How many moves ahead the computer will search.\nFewer moves is faster and easier, while more moves is slower and more difficult.");
            }
        });
        if depth as u8 != model.ai_search_depth {
            insert_if_empty(&mut event, Event::SetAIDepth(depth as u8));
        }
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

            if let Some(click) = board(model, Vec2::new(size.0 - 16.0, size.1 - 170.0)) {
                insert_if_empty(&mut event, click);
            }

            let display_vitals = || {
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
            };

            let button_size = Vec2::new(120.0, 20.0);
            use model::Outcome::*;
            match model.outcome {
                Win(color) => {
                    ui.text(format!("{:?} wins!", color));
                    display_vitals();
                    if model.can_undo() && ui.button(im_str!("Undo"), button_size) {
                        insert_if_empty(&mut event, Event::Undo);
                    }
                }
                InProgress => {
                    ui.text(format!("It's {:?}'s turn.", model.board.turn));
                    display_vitals();

                    horz_button_layout(
                        ui,
                        vec![
                            (model.can_undo(), im_str!("Undo"), Event::Undo),
                            (model.can_redo(), im_str!("Redo"), Event::Redo),
                        ],
                        &button_size,
                        &mut event,
                    );
                    horz_button_layout(
                        ui,
                        vec![
                            (true, im_str!("Resign"), Event::Resign),
                            (
                                model.board.can_exchange()
                                    && model.current_player() == Player::Human,
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
                }
                // Draw cases
                _ => {
                    let message = match model.outcome {
                        DrawStalemate => "It's a draw by stalemate!",
                        DrawThreefoldRepetition => "It's a draw by threefold repetition!",
                        DrawInsufficientMaterial => "It's a draw by insufficient material!",
                        _ => unreachable!(),
                    };
                    ui.text(message);
                    display_vitals();
                    if model.can_undo() && ui.button(im_str!("Undo"), button_size) {
                        insert_if_empty(&mut event, Event::Undo);
                    }
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
    if !buttons.iter().any(|&(show, _, _)| show) {
        return;
    }

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
