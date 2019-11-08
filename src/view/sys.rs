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
use imgui::{Context, FontConfig, FontSource, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use crate::model::Model;
use crate::update;

const FRAME_DURATION: Duration = Duration::from_millis(16);

pub fn run<F: FnMut(&mut Model, &Ui, [f32; 2]) -> bool>(
    title: String,
    dimensions: (u32, u32),
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

    let display =
        Display::new(window, context, &events_loop).expect("Could not initialize display");
    let gl_window = display.gl_window();
    let window = gl_window.window();

    let mut ctx = Context::create();
    ctx.style_mut().use_classic_colors();
    ctx.set_ini_filename(None);

    let mut platform = WinitPlatform::init(&mut ctx);
    platform.attach_window(ctx.io_mut(), &gl_window.window(), HiDpiMode::Rounded);

    let hidpi_factor = platform.hidpi_factor();
    ctx.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../../assets/FiraSans-Regular.ttf"),
        size_pixels: (21.0 * hidpi_factor) as f32,
        config: Some(FontConfig {
            oversample_h: 4,
            oversample_v: 4,
            rasterizer_multiply: 1.05,
            ..FontConfig::default()
        }),
    }]);
    ctx.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let mut renderer = Renderer::init(&mut ctx, &display).expect("Failed to initialize renderer");

    let mut last_frame = Instant::now();

    let mut render = |model: &mut Model,
                      ctx: &mut Context,
                      platform: &mut WinitPlatform,
                      last_frame: &mut Instant| {
        let display_size = {
            let io = ctx.io_mut();
            platform
                .prepare_frame(io, &window)
                .expect("Failed to start frame");
            *last_frame = io.update_delta_time(*last_frame);
            io.display_size
        };

        let ui = ctx.frame();
        if !run_ui(model, &ui, display_size) {
            return false;
        }

        let mut target = display.draw();
        target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
        platform.prepare_render(&ui, &window);
        renderer
            .render(&mut target, ui.render())
            .expect("Rendering failed");
        target.finish().expect("Failed to swap buffers");
        true
    };

    // Render one frame before the event loop so the screen isn't empty
    render(&mut model, &mut ctx, &mut platform, &mut last_frame);

    events_loop.run_forever(|event| {
        use glium::glutin::WindowEvent::*;
        use glium::glutin::{ControlFlow, Event, MouseButton, TouchPhase, VirtualKeyCode};
        platform.handle_event(ctx.io_mut(), &window, &event);

        if let Event::Awakened = event {
            if Instant::now() - last_frame < FRAME_DURATION {
                // Receive the AI move, and queue the next one (if it's a computer-only game)
                update::update(&mut model, None);
                update::update(&mut model, None);

                // If the AI is moving very quickly, then the last move of the game will be
                // throttled and not receive a render. This appears to "freeze" the game. So, we
                // render if the game is finished.
                if model.is_game_over()
                    && !render(&mut model, &mut ctx, &mut platform, &mut last_frame)
                {
                    return ControlFlow::Break;
                }
            } else {
                // Receive the AI move, then render
                update::update(&mut model, None);
                if !render(&mut model, &mut ctx, &mut platform, &mut last_frame) {
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
                    if !render(&mut model, &mut ctx, &mut platform, &mut last_frame) {
                        return ControlFlow::Break;
                    }
                }
                CursorMoved { .. } => {
                    if Instant::now() - last_frame < FRAME_DURATION {
                        return ControlFlow::Continue;
                    } else if !render(&mut model, &mut ctx, &mut platform, &mut last_frame) {
                        return ControlFlow::Break;
                    }
                }
                MouseWheel {
                    phase: TouchPhase::Moved,
                    ..
                } => {
                    if !render(&mut model, &mut ctx, &mut platform, &mut last_frame) {
                        return ControlFlow::Break;
                    }
                }
                MouseInput { button, .. } => {
                    if MouseButton::Left == button {
                        // Render twice to immediately show the results of the click
                        if !render(&mut model, &mut ctx, &mut platform, &mut last_frame) {
                            return ControlFlow::Break;
                        }
                        if !render(&mut model, &mut ctx, &mut platform, &mut last_frame) {
                            return ControlFlow::Break;
                        }
                    }
                }
                Touch(glutin::Touch {
                    phase, location, ..
                }) => {
                    let io = ctx.io_mut();
                    let pos = platform.scale_pos_from_winit(&window, location);
                    io.mouse_pos = [pos.x as f32, pos.y as f32];
                    io.mouse_down[0] = phase == TouchPhase::Started || phase == TouchPhase::Moved;

                    match phase {
                        TouchPhase::Moved => {
                            if Instant::now() - last_frame < FRAME_DURATION {
                                return ControlFlow::Continue;
                            } else if !render(&mut model, &mut ctx, &mut platform, &mut last_frame)
                            {
                                return ControlFlow::Break;
                            }
                        }
                        _ => {
                            // Render twice to immediately show the results of the touch
                            if !render(&mut model, &mut ctx, &mut platform, &mut last_frame) {
                                return ControlFlow::Break;
                            }
                            if !render(&mut model, &mut ctx, &mut platform, &mut last_frame) {
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
