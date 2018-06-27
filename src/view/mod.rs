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

use imgui::{ImGuiCond, ImStr, StyleVar, Ui};

use self::board::board;
pub use self::sys::run;
use self::vec2::Vec2;
use model::{Color, ColorMap, GameType, Model, Player};
use update::Event;

pub fn draw(ui: &Ui, size: (f32, f32), model: &Model) -> Option<Event> {
    let mut event = None;

    ui.main_menu_bar(|| {
        ui.menu(im_str!("Game")).build(|| {
            ui.menu_item(im_str!("New game")).enabled(false).build();

            ui.menu(im_str!("Laurentius")).build(|| {
                player_options(ui, &mut event, GameType::Laurentius);
            });
            ui.menu(im_str!("Ocius")).build(|| {
                player_options(ui, &mut event, GameType::Ocius);
            });

            ui.separator();

            if ui.menu_item(im_str!("Quit")).build() {
                insert_if_empty(&mut event, Event::Quit);
            }
        });

        ui.menu(im_str!("Computer")).build(|| {
            ui.slider_int(
                im_str!("Search depth"),
                &mut model.ai_search_depth.borrow_mut(),
                1,
                7,
            ).build();
            if ui.is_item_hovered() {
                ui.tooltip_text("How many moves ahead the computer will search.\nFewer moves is faster and easier, while more moves is slower and more difficult.");
            }

            ui.menu_item(im_str!("Show debug info"))
                .selected(&mut model.show_ai_debug_window.borrow_mut())
                .build();
        });
    });

    ui.with_style_var(StyleVar::WindowRounding(0.0), || {
        draw_window(ui, size, model, &mut event);
    });

    if *model.show_ai_debug_window.borrow() {
        ui.window(im_str!("AI Debug Info"))
            .opened(&mut model.show_ai_debug_window.borrow_mut())
            .size((300.0, 600.0), ImGuiCond::FirstUseEver)
            .build(|| {
                if let Ok(debug_info) = model.ai.debug_info.read() {
                    ui.text(debug_info.clone());
                }
            });
    }

    event
}

fn player_options(ui: &Ui, event: &mut Option<Event>, game_type: GameType) {
    use self::Player::*;
    if ui.menu_item(im_str!("Human vs. Human")).build() {
        insert_if_empty(
            event,
            Event::NewGame(game_type, ColorMap::new(Human, Human)),
        );
    }
    if ui.menu_item(im_str!("Human vs. Computer")).build() {
        insert_if_empty(
            event,
            Event::NewGame(game_type, ColorMap::new(Human, Computer)),
        );
    }
    if ui.menu_item(im_str!("Computer vs. Human")).build() {
        insert_if_empty(
            event,
            Event::NewGame(game_type, ColorMap::new(Computer, Human)),
        );
    }
    if ui.menu_item(im_str!("Computer vs. Computer")).build() {
        insert_if_empty(
            event,
            Event::NewGame(game_type, ColorMap::new(Computer, Computer)),
        );
    }
}

fn draw_window(ui: &Ui, size: (f32, f32), model: &Model, event: &mut Option<Event>) {
    ui.window(im_str!("Coerceo"))
        .size(size, ImGuiCond::Always)
        .position((0.0, 27.0), ImGuiCond::Always)
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .no_bring_to_front_on_focus(true)
        .build(|| {
            ui.text("Welcome to Coerceo!");
            ui.text(format!(
                "{:?} vs. {:?}",
                model.players.white, model.players.black
            ));

            if let Some(click) = board(ui, model, Vec2::new(size.0 - 16.0, size.1 - 232.0)) {
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

            let button_size = Vec2::new(155.0, 29.0);
            use model::Outcome::*;
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
    size: Vec2,
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
