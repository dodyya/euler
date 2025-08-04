use crate::sim::Simulation;
use hsv::{self, hsv_to_rgb};
use pixels::{Pixels, SurfaceTexture};
use std::cmp::min;
use std::time::{Duration, Instant};
use winit::event::{ElementState, VirtualKeyCode};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct Visualization {
    color_mode: ColorMode,
    vis_mode: VisualizationMode,
    pixel_scale: u32,
    window: Window,
    pixels: Pixels,
    sim: Simulation,
    event_loop: EventLoop<()>,
}

const TIME_DETAILS: bool = true;
const DETAILS_ON_CLICK: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorMode {
    Rgb,
    Grayscale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisualizationMode {
    Smoke,
    Pressure,
    Speed,
    SmokePressure,
    SmokeSpeed,
}

impl Visualization {
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_scale = min(1864 / height, 2880 / width);
        let event_loop = EventLoop::new();
        let physical_size = PhysicalSize::new(width * pixel_scale, height * pixel_scale);

        let window = WindowBuilder::new()
            .with_title("Eulerian Fluid Simulation")
            .with_inner_size(physical_size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap();

        let surface_texture =
            SurfaceTexture::new(physical_size.width, physical_size.height, &window);

        let pixels = Pixels::new(width, height, surface_texture).unwrap();
        let sim = Simulation::new(width as usize, height as usize);

        Visualization {
            color_mode: ColorMode::Rgb,
            vis_mode: VisualizationMode::SmokePressure,
            pixel_scale,
            window,
            pixels,
            sim,
            event_loop,
        }
    }

    pub fn run(mut self) {
        let mut cursor_position: Option<(f64, f64)> = None;
        let mut last_frame_start = Instant::now();
        let mut frame_time = Duration::ZERO;
        let mut ticker: u8 = 0;
        let mut step_time: f64 = 0.0;
        let mut mouse_down = false;

        self.event_loop.run(move |event, _, control_flow| {
            if ticker == 0 {
                self.window.set_title(&format!(
                    "Eulerian Fluid Simulation: {:?} mode - {:?} - FPS: {:.0}",
                    self.color_mode,
                    self.vis_mode,
                    1.0 / frame_time.as_secs_f64() as f64
                ));
                if TIME_DETAILS {
                    println!(
                        "Time for render: {}",
                        last_frame_start.elapsed().as_secs_f64() - step_time
                    );
                }
            }
            ticker = ticker.wrapping_add(8);

            use ColorMode as cm;
            use VisualizationMode as vm;
            let imag_buffer = match self.vis_mode {
                vm::Pressure | vm::SmokePressure => self.sim.get_pressure(),
                vm::Speed | vm::SmokeSpeed => &self.sim.get_speed(),
                vm::Smoke => self.sim.get_smoke(),
            };
            let mask = match self.vis_mode {
                vm::Pressure | vm::Speed | vm::Smoke => self.sim.get_s(),
                vm::SmokeSpeed | vm::SmokePressure => self.sim.get_smoke(),
            };
            render(self.pixels.frame_mut(), imag_buffer, mask, self.color_mode);

            _ = self.pixels.render();

            frame_time = last_frame_start.elapsed();
            last_frame_start = Instant::now();

            self.sim.step();
            // self.sim.step(frame_time.as_secs_f64() * 3.0);

            if TIME_DETAILS && ticker == 0 {
                step_time = last_frame_start.elapsed().as_secs_f64();
                println!("Time for simulation step: {}", step_time);
            }

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::MouseInput {
                        state: winit::event::ElementState::Pressed,
                        button: winit::event::MouseButton::Left,
                        ..
                    } => {
                        if let Some(cursor_pos) = cursor_position {
                            let grid_x = (cursor_pos.0 / self.pixel_scale as f64) as i32;
                            let grid_y = (cursor_pos.1 / self.pixel_scale as f64) as i32;
                            if DETAILS_ON_CLICK {
                                self.sim.cell_info(grid_x as usize, grid_y as usize);
                            } else {
                                mouse_down = true;
                            }
                        }
                    }

                    WindowEvent::MouseInput {
                        state: winit::event::ElementState::Released,
                        button: winit::event::MouseButton::Left,
                        ..
                    } => {
                        mouse_down = false;
                    }

                    WindowEvent::MouseInput {
                        state: winit::event::ElementState::Pressed,
                        button: winit::event::MouseButton::Right,
                        ..
                    } => {
                        if let Some(cursor_pos) = cursor_position {
                            let grid_x = (cursor_pos.0 / self.pixel_scale as f64) as i32;
                            let grid_y = (cursor_pos.1 / self.pixel_scale as f64) as i32;
                            self.sim.cell_info(grid_x as usize, grid_y as usize);
                        }
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        cursor_position = Some((position.x, position.y));
                        if mouse_down {
                            let cursor_pos = cursor_position.unwrap();
                            let grid_x = (cursor_pos.0 / self.pixel_scale as f64) as i32;
                            let grid_y = (cursor_pos.1 / self.pixel_scale as f64) as i32;
                            self.sim.draw_filled_circle(grid_x, grid_y, 2.5);
                        }
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.state != ElementState::Pressed {
                            return;
                        }
                        if let Some(key) = input.virtual_keycode {
                            match key {
                                VirtualKeyCode::Space => {
                                    self.sim.reset_except_walls();
                                    ticker = 0;
                                }
                                VirtualKeyCode::R => {
                                    self.sim.reset();
                                    ticker = 0;
                                }
                                VirtualKeyCode::Left => {
                                    self.vis_mode = match self.vis_mode {
                                        vm::Pressure => vm::Smoke,
                                        vm::Smoke => vm::Speed,
                                        vm::Speed => vm::SmokePressure,
                                        vm::SmokePressure => vm::SmokeSpeed,
                                        vm::SmokeSpeed => vm::Pressure,
                                    };
                                    ticker = 0;
                                }
                                VirtualKeyCode::Right => {
                                    self.vis_mode = match self.vis_mode {
                                        vm::Smoke => vm::Pressure,
                                        vm::Speed => vm::Smoke,
                                        vm::SmokePressure => vm::Speed,
                                        vm::SmokeSpeed => vm::SmokePressure,
                                        vm::Pressure => vm::SmokeSpeed,
                                    };
                                    ticker = 0;
                                }
                                VirtualKeyCode::Up | VirtualKeyCode::Down => {
                                    self.color_mode = match self.color_mode {
                                        cm::Rgb => cm::Grayscale,
                                        cm::Grayscale => cm::Rgb,
                                    };
                                    ticker = 0;
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },

                _ => {}
            }
        });
    }
}

fn render(frame: &mut [u8], imag: &[f64], mask: &[f64], cm: ColorMode) {
    //Go from [0.0, 1.0] to [0,255]; then assign that value to R, G, B and make A=255.
    // dbg!(frame.len());
    let min = imag.iter().fold(f64::MAX, |acc, &x| acc.min(x));
    let max = imag.iter().fold(f64::MIN, |acc, &x| acc.max(x));
    let range = (max - min).max(0.000001);

    let buffer: Vec<u8> = imag
        .iter()
        .map(|x| (x - min) / (range))
        .zip(mask.iter())
        .map(|(y, &m)| match cm {
            ColorMode::Rgb => hsv_to_rgb(
                (if y.is_nan() { 1.0 } else { y.clamp(0.0, 1.0) }) * 300.0,
                range.clamp(0.5, 1.0),
                m.clamp(0.0, 1.0),
            ),
            ColorMode::Grayscale => (
                ((if y.is_nan() { 1.0 } else { y.clamp(0.0, 1.0) }) * m * 255.0) as u8,
                ((if y.is_nan() { 1.0 } else { y.clamp(0.0, 1.0) }) * m * 255.0) as u8,
                ((if y.is_nan() { 1.0 } else { y.clamp(0.0, 1.0) }) * m * 255.0) as u8,
            ),
        })
        .map(|(r, g, b)| [r as u8, g as u8, b as u8, 255])
        .flatten()
        .collect();
    frame.copy_from_slice(&buffer);
}
