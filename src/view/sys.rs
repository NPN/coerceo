/*
 * Copyright (C) 2015-2017 The imgui-rs Developers
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

use std::time::{Duration, Instant};

use glium::glutin::{self, Api, GlRequest};
use glium::{Display, Surface};
use imgui::{FontGlyphRange, FrameSize, ImFontConfig, ImGui, Ui};
use imgui_glium_renderer::Renderer;

use crate::model::Model;
use crate::update;

const FRAME_DURATION: Duration = Duration::from_millis(16);

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (f64, f64),
    pressed: (bool, bool, bool),
    wheel: f32,
}

pub fn run<F: FnMut(&mut Model, &Ui, (f32, f32)) -> bool>(
    title: String,
    dimensions: (u32, u32),
    clear_color: [f32; 4],
    mut events_loop: glutin::EventsLoop,
    mut model: Model,
    mut run_ui: F,
) {
    let window = glutin::WindowBuilder::new()
        .with_title(title)
        .with_dimensions(dimensions.into());
    let mut context = glutin::ContextBuilder::new().with_vsync(true);
    if cfg!(target_os = "android") {
        // https://github.com/tomaka/android-rs-glue/issues/153#issuecomment-318348732
        // On Android we must specify an OpenGL ES version or glutin will assume we are using an
        // unsupported version and panic
        context = context.with_gl(GlRequest::Specific(Api::OpenGlEs, (2, 0)));
    }

    let display = Display::new(window, context, &events_loop).unwrap();
    let gl_window = display.gl_window();

    let mut imgui = ImGui::init();
    unsafe {
        imgui::sys::igStyleColorsClassic(imgui.style_mut());
    }
    imgui.set_ini_filename(None);

    let config = ImFontConfig::new()
        .oversample_h(4)
        .oversample_v(4)
        .size_pixels(21.0)
        .rasterizer_multiply(1.05);

    config.add_font(
        &mut imgui.fonts(),
        include_bytes!("../../assets/FiraSans-Regular.ttf"),
        &FontGlyphRange::default(),
    );

    let mut renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

    let mut last_frame = Instant::now();
    let mut mouse_state = MouseState::default();

    let mut render = |model: &mut Model, imgui: &mut ImGui, last_frame: &mut Instant| {
        let now = Instant::now();
        let delta = now - *last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        *last_frame = now;

        let frame_size = FrameSize {
            logical_size: gl_window.get_inner_size().unwrap().into(),
            hidpi_factor: gl_window.get_hidpi_factor(),
        };

        let ui = imgui.frame(frame_size, delta_s);
        if !run_ui(
            model,
            &ui,
            (
                frame_size.logical_size.0 as f32,
                frame_size.logical_size.1 as f32,
            ),
        ) {
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
    render(&mut model, &mut imgui, &mut last_frame);

    events_loop.run_forever(|event| {
        use glium::glutin::ElementState::Pressed;
        use glium::glutin::WindowEvent::*;
        use glium::glutin::{
            ControlFlow, Event, MouseButton, MouseScrollDelta, TouchPhase, VirtualKeyCode,
        };

        if let Event::Awakened = event {
            if Instant::now() - last_frame < FRAME_DURATION {
                // Receive the AI move, and queue the next one (if it's a computer-only game)
                update::update(&mut model, None);
                update::update(&mut model, None);

                // If the AI is moving very quickly, then the last move of the game will be
                // throttled and not receive a render. This appears to "freeze" the game. So, we
                // render if the game is finished.
                if model.is_game_over() && !render(&mut model, &mut imgui, &mut last_frame) {
                    return ControlFlow::Break;
                }
            } else {
                // Receive the AI move, then render
                update::update(&mut model, None);
                if !render(&mut model, &mut imgui, &mut last_frame) {
                    return ControlFlow::Break;
                }
            }
        } else if let Event::Suspended(true) = event {
            // This is so that the AI doesn't run in the background on Android. Technically, we
            // should also call update or render on Suspended(false) to restart the AI, but there's
            // no point since the app crashes when it's resumed or even right after it's suspended.
            model.ai.stop();
        } else if let Event::WindowEvent { event, .. } = event {
            match event {
                CloseRequested => return ControlFlow::Break,
                KeyboardInput { input, .. } => {
                    if let Some(VirtualKeyCode::Q) = input.virtual_keycode {
                        if cfg!(target_os = "macos") && input.modifiers.logo {
                            return ControlFlow::Break;
                        }
                    }
                }
                Refresh | Resized(_) | HiDpiFactorChanged(_) => {
                    if !render(&mut model, &mut imgui, &mut last_frame) {
                        return ControlFlow::Break;
                    }
                }
                CursorMoved { position, .. } => {
                    mouse_state.pos = position.into();
                    update_mouse(&mut imgui, &mut mouse_state);

                    if Instant::now() - last_frame < FRAME_DURATION {
                        return ControlFlow::Continue;
                    } else if !render(&mut model, &mut imgui, &mut last_frame) {
                        return ControlFlow::Break;
                    }
                }
                MouseWheel {
                    delta,
                    phase: TouchPhase::Moved,
                    ..
                } => {
                    mouse_state.wheel = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y,
                        MouseScrollDelta::PixelDelta(position) => position.y as f32,
                    };
                    update_mouse(&mut imgui, &mut mouse_state);

                    if !render(&mut model, &mut imgui, &mut last_frame) {
                        return ControlFlow::Break;
                    }
                }
                MouseInput { state, button, .. } => {
                    if MouseButton::Left == button {
                        mouse_state.pressed.0 = state == Pressed;
                        update_mouse(&mut imgui, &mut mouse_state);

                        // Render twice to immediately show the results of the click
                        if !render(&mut model, &mut imgui, &mut last_frame) {
                            return ControlFlow::Break;
                        }
                        if !render(&mut model, &mut imgui, &mut last_frame) {
                            return ControlFlow::Break;
                        }
                    }
                }
                Touch(glutin::Touch {
                    phase, location, ..
                }) => {
                    mouse_state.pos = location.into();
                    mouse_state.pressed.0 =
                        phase == TouchPhase::Started || phase == TouchPhase::Moved;
                    update_mouse(&mut imgui, &mut mouse_state);

                    match phase {
                        TouchPhase::Moved => {
                            if Instant::now() - last_frame < FRAME_DURATION {
                                return ControlFlow::Continue;
                            } else if !render(&mut model, &mut imgui, &mut last_frame) {
                                return ControlFlow::Break;
                            }
                        }
                        _ => {
                            // Render twice to immediately show the results of the touch
                            if !render(&mut model, &mut imgui, &mut last_frame) {
                                return ControlFlow::Break;
                            }
                            if !render(&mut model, &mut imgui, &mut last_frame) {
                                return ControlFlow::Break;
                            }
                        }
                    }
                }
                _ => (),
            }
        }
        ControlFlow::Continue
    });
}

fn update_mouse(imgui: &mut ImGui, mouse_state: &mut MouseState) {
    imgui.set_mouse_pos(mouse_state.pos.0 as f32, mouse_state.pos.1 as f32);
    imgui.set_mouse_down([
        mouse_state.pressed.0,
        mouse_state.pressed.1,
        mouse_state.pressed.2,
        false,
        false,
    ]);
    imgui.set_mouse_wheel(mouse_state.wheel);
    mouse_state.wheel = 0.0;
}
