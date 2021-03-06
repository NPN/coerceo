/*
 * Copyright (C) 2017-2019 Ryan Huang
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

use imgui::{Condition, ImStr, MenuItem, Slider, StyleVar, Ui, Window};

use self::board::board;
pub use self::sys::run;
use self::vec2::Vec2;
use crate::model::{Color, ColorMap, GameType, Model, Player};
use crate::update::Event;

pub fn draw(ui: &Ui, size: [f32; 2], model: &Model) -> Option<Event> {
    let mut event = None;
    let mut window_states = model.window_states.borrow_mut();

    ui.main_menu_bar(|| {
        ui.menu(im_str!("Game"), true, || {
            MenuItem::new(im_str!("New game")).enabled(false).build(ui);

            ui.menu(im_str!("Laurentius"), true, || {
                player_options(ui, &mut event, GameType::Laurentius);
            });
            ui.menu(im_str!("Ocius"), true, || {
                player_options(ui, &mut event, GameType::Ocius);
            });

            ui.separator();

            MenuItem::new(im_str!("Rules")).enabled(false).build(ui);
            if ui.is_item_hovered() {
                ui.tooltip_text("Any changes to the rules apply at the start of the next game.");
            }

            MenuItem::new(im_str!("One tile to exchange"))
                .build_with_ref(ui, &mut model.exchange_one_hex.borrow_mut());
            if ui.is_item_hovered() {
                ui.tooltip_text(
                    "If selected, only one tile (rather than two) is needed to exchange for a piece."
                );
            }

            ui.separator();

            if MenuItem::new(im_str!("Quit")).build(ui) {
                insert_if_empty(&mut event, Event::Quit);
            }
        });

        ui.menu(im_str!("Computer"), true, || {
            Slider::new(im_str!("Search depth"), 1..=7)
                .build(ui, &mut model.ai_search_depth.borrow_mut());
            if ui.is_item_hovered() {
                ui.tooltip_text(
                    "How many moves ahead the computer will search.\nFewer moves is \
                     faster and easier, while more moves is slower and more difficult.",
                );
            }

            MenuItem::new(im_str!("Show debug info")).build_with_ref(ui, &mut window_states.ai_debug);
        });

        ui.menu(im_str!("Help"), true, || {
            MenuItem::new(im_str!("How to Play")).build_with_ref(ui, &mut window_states.how_to_play);
            MenuItem::new(im_str!("About")).build_with_ref(ui, &mut window_states.about);
        });
    });

    let token = ui.push_style_var(StyleVar::WindowRounding(0.0));
    draw_window(ui, size, model, &mut event);
    token.pop(ui);

    if window_states.ai_debug {
        Window::new(im_str!("AI Debug Info"))
            .opened(&mut window_states.ai_debug)
            .size([300.0, 600.0], Condition::FirstUseEver)
            .build(ui, || {
                if let Ok(debug_info) = model.ai.debug_info.read() {
                    ui.text(debug_info.clone());
                }
            });
    }

    if window_states.how_to_play {
        // TODO: Create an interactive, in-game tutorial to teach the rules of the game
        Window::new(im_str!("How to Play"))
            .opened(&mut window_states.how_to_play)
            .build(ui, || {
                ui.text(
                    "Unfortunately, there isn't an in-game tutorial. Sorry!\nSee coerceo.com for \
                     the rules of the game.",
                );
            });
    }

    if window_states.about {
        Window::new(im_str!("About"))
            .opened(&mut window_states.about)
            .build(ui, || {
                ui.text(
                    "Coerceo v1.0.0 (https://github.com/NPN/coerceo)

An unofficial clone of a strategic board game.

Copyright (C) 2017-2019 Ryan Huang
This program comes with absolutely no warranty.
See the GNU AGPL, version 3 or later, for details.

All rights, trademarks, copyrights, concepts, etc. of the game
Coerceo belong to the Coerceo Company.

This program includes work from the following sources:

imgui-rs (https://github.com/Gekkio/imgui-rs)
Copyright (c) 2015-2017 The imgui-rs Developers
Licensed under the MIT License.

Fira Sans (https://github.com/mozilla/Fira)
Copyright (c) 2012-2015 The Mozilla Foundation and Telefonica S.A.
Licensed under the SIL Open Font License v1.1",
                );
            });
    }

    event
}

fn player_options(ui: &Ui, event: &mut Option<Event>, game_type: GameType) {
    use self::Player::*;
    if MenuItem::new(im_str!("Human vs. Human")).build(ui) {
        insert_if_empty(
            event,
            Event::NewGame(game_type, ColorMap::new(Human, Human)),
        );
    }
    if MenuItem::new(im_str!("Human vs. Computer")).build(ui) {
        insert_if_empty(
            event,
            Event::NewGame(game_type, ColorMap::new(Human, Computer)),
        );
    }
    if MenuItem::new(im_str!("Computer vs. Human")).build(ui) {
        insert_if_empty(
            event,
            Event::NewGame(game_type, ColorMap::new(Computer, Human)),
        );
    }
    if MenuItem::new(im_str!("Computer vs. Computer")).build(ui) {
        insert_if_empty(
            event,
            Event::NewGame(game_type, ColorMap::new(Computer, Computer)),
        );
    }
}

fn draw_window(ui: &Ui, size: [f32; 2], model: &Model, event: &mut Option<Event>) {
    Window::new(im_str!("Coerceo"))
        .size(size, Condition::Always)
        .position([0.0, 27.0], Condition::Always)
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .bring_to_front_on_focus(false)
        .build(ui, || {
            ui.text("Welcome to Coerceo!");

            let exchange_hex_string = if model.board.hexes_to_exchange == 1 {
                "One tile to exchange"
            } else {
                "Two tiles to exchange"
            };
            ui.text(format!(
                "{:?} vs. {:?} ({})",
                model.players.white, model.players.black, exchange_hex_string
            ));

            let board_size = Vec2::new((size[0] - 16.0).max(100.0), (size[1] - 232.0).max(100.0));
            if let Some(click) = board(ui, model, board_size) {
                insert_if_empty(event, click);
            }

            let format_piece_count = |count| match count {
                1 => String::from("1 piece"),
                _ => format!("{} pieces", count),
            };

            let format_hex_count = |count| match count {
                1 => String::from("1 captured tile"),
                _ => format!("{} captured tiles", count),
            };

            let display_vitals = || {
                ui.text(format!(
                    "{:?} has {} and {}.",
                    Color::White,
                    format_piece_count(model.board.pieces(Color::White)),
                    format_hex_count(model.board.hexes(Color::White)),
                ));
                ui.text(format!(
                    "{:?} has {} and {}.",
                    Color::Black,
                    format_piece_count(model.board.pieces(Color::Black)),
                    format_hex_count(model.board.hexes(Color::Black)),
                ));
            };

            let button_size = [155.0, 29.0];
            use crate::model::Outcome::*;
            match model.outcome {
                Win(color) => {
                    ui.text(format!("{:?} wins!", color));
                    display_vitals();
                    if model.can_undo() && ui.button(im_str!("Undo"), button_size) {
                        insert_if_empty(event, Event::Undo);
                    }
                }
                InProgress => {
                    if model.players.white == model.players.black {
                        ui.text(format!("It's {:?}'s turn.", model.board.turn,));
                    } else {
                        ui.text(match model.current_player() {
                            Player::Computer => "Waiting for the computer...",
                            Player::Human => "It's your turn.",
                        });
                    }

                    display_vitals();

                    horz_button_layout(
                        ui,
                        vec![
                            (model.can_undo(), im_str!("Undo"), Event::Undo),
                            (model.can_redo(), im_str!("Redo"), Event::Redo),
                        ],
                        button_size,
                        event,
                    );
                    let is_human_player = model.current_player() == Player::Human;
                    horz_button_layout(
                        ui,
                        vec![
                            (is_human_player, im_str!("Resign"), Event::Resign),
                            (
                                model.board.can_exchange() && is_human_player,
                                if model.exchanging {
                                    im_str!("Stop Exchanging")
                                } else {
                                    im_str!("Exchange")
                                },
                                Event::Exchange,
                            ),
                        ],
                        button_size,
                        event,
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
                        insert_if_empty(event, Event::Undo);
                    }
                }
            }
        });
}

fn horz_button_layout(
    ui: &Ui,
    buttons: Vec<(bool, &ImStr, Event)>,
    size: [f32; 2],
    event: &mut Option<Event>,
) {
    if !buttons.iter().any(|&(show, _, _)| show) {
        return;
    }

    for (show, label, action) in buttons {
        if show {
            if ui.button(label, size) {
                insert_if_empty(event, action);
            }
        } else {
            ui.dummy(size);
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
