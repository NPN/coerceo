/*
 * Copyright (C) 2015-2017 The imgui-rs Developers
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

use std::time::{Duration, Instant};

use glium::glutin;
use glium::{Display, Surface};
use imgui::{ImGui, Ui};
use imgui_glium_renderer::Renderer;

const FRAME_DURATION: Duration = Duration::from_millis(50);

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

pub fn run<F: FnMut(&Ui, (f32, f32)) -> bool>(
    title: String,
    dimensions: (u32, u32),
    clear_color: [f32; 4],
    mut events_loop: glutin::EventsLoop,
    mut run_ui: F,
) {
    let window = glutin::WindowBuilder::new()
        .with_title(title)
        .with_dimensions(dimensions.0, dimensions.1);
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let display = Display::new(window, context, &events_loop).unwrap();

    let mut imgui = ImGui::init();
    let mut renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

    imgui.set_ini_filename(None);

    let mut last_frame = Instant::now();
    let mut mouse_state = MouseState::default();

    let mut render = |imgui: &mut ImGui, last_frame: &mut Instant| {
        let now = Instant::now();
        let delta = now - *last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        *last_frame = now;

        let gl_window = display.gl_window();
        let size_pixels = gl_window.get_inner_size().unwrap();
        let hidpi = gl_window.hidpi_factor();
        let size_points = (
            (size_pixels.0 as f32 / hidpi) as u32,
            (size_pixels.1 as f32 / hidpi) as u32,
        );

        let ui = imgui.frame(size_points, size_pixels, delta_s);
        if !run_ui(&ui, (size_points.0 as f32, size_points.1 as f32)) {
            return false;
        }

        let mut target = display.draw();
        target.clear_color(
            clear_color[0],
            clear_color[1],
            clear_color[2],
            clear_color[3],
        );
        renderer.render(&mut target, ui).expect("Rendering failed");
        target.finish().unwrap();
        true
    };

    // Render one frame before the event loop so the screen isn't empty
    render(&mut imgui, &mut last_frame);

    events_loop.run_forever(|event| {
        use glium::glutin::ElementState::Pressed;
        use glium::glutin::WindowEvent::*;
        use glium::glutin::{ControlFlow, Event, MouseButton, VirtualKeyCode};

        if let Event::Awakened = event {
            // Render twice to immediately show the new AI move
            if !render(&mut imgui, &mut last_frame) {
                return ControlFlow::Break;
            }
            if !render(&mut imgui, &mut last_frame) {
                return ControlFlow::Break;
            }
        } else if let Event::WindowEvent { event, .. } = event {
            match event {
                Closed => return ControlFlow::Break,
                KeyboardInput { input, .. } => {
                    if let Some(VirtualKeyCode::Q) = input.virtual_keycode {
                        if cfg!(target_os = "macos") && input.modifiers.logo {
                            return ControlFlow::Break;
                        }
                    }
                }
                Refresh | Resized(_, _) | HiDPIFactorChanged(_) => {
                    if !render(&mut imgui, &mut last_frame) {
                        return ControlFlow::Break;
                    }
                }
                CursorMoved {
                    position: (x, y), ..
                } => {
                    mouse_state.pos = (x as i32, y as i32);
                    update_mouse(&mut imgui, &mut mouse_state);

                    if Instant::now() - last_frame < FRAME_DURATION {
                        return ControlFlow::Continue;
                    } else if !render(&mut imgui, &mut last_frame) {
                        return ControlFlow::Break;
                    }
                }
                MouseInput { state, button, .. } => if MouseButton::Left == button {
                    mouse_state.pressed.0 = state == Pressed;
                    update_mouse(&mut imgui, &mut mouse_state);

                    // Render twice to immediately show the results of the click
                    if !render(&mut imgui, &mut last_frame) {
                        return ControlFlow::Break;
                    }
                    if !render(&mut imgui, &mut last_frame) {
                        return ControlFlow::Break;
                    }
                },
                _ => (),
            }
        }
        ControlFlow::Continue
    });
}

fn update_mouse(imgui: &mut ImGui, mouse_state: &mut MouseState) {
    let scale = imgui.display_framebuffer_scale();
    imgui.set_mouse_pos(
        mouse_state.pos.0 as f32 / scale.0,
        mouse_state.pos.1 as f32 / scale.1,
    );
    imgui.set_mouse_down(&[
        mouse_state.pressed.0,
        mouse_state.pressed.1,
        mouse_state.pressed.2,
        false,
        false,
    ]);
    imgui.set_mouse_wheel(mouse_state.wheel / scale.1);
    mouse_state.wheel = 0.0;
}
