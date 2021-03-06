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

use glium::glutin::EventsLoop;
use imgui::Ui;

use coerceo::{
    model::{ColorMap, GameType, Model, Player},
    update, view,
};

fn main() {
    let events_loop = EventsLoop::new();
    let events_proxy = events_loop.create_proxy();

    let model = Model::new(
        GameType::Laurentius,
        ColorMap::new(Player::Human, Player::Human),
        events_proxy,
    );

    view::run(
        String::from("Coerceo"),
        (800, 800),
        events_loop,
        model,
        |model, ui, size| game_loop(model, ui, size),
    );
}

fn game_loop(model: &mut Model, ui: &Ui, size: [f32; 2]) -> bool {
    let event = view::draw(ui, size, model);
    update::update(model, event)
}
