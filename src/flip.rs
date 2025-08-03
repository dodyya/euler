use crate::sim::Simulation;
use hsv::{self, hsv_to_rgb};
use pixels::{Pixels, SurfaceTexture};
use std::cmp::min;
use std::thread;
use std::time::{Duration, Instant};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct Flip {
    width: u32,
    height: u32,
    pixel_scale: u32,
    window: Window,
    pixels: Pixels,
    sim: Simulation,
    event_loop: EventLoop<()>,
    palette: (f64, f64), //(range, offset)
}
const RGB: bool = false;
const TIME_DETAILS: bool = false;
const DRAW_PRESSURE: bool = false;
const DRAW_U: bool = false;

const DETAILS_ON_CLICK: bool = true;

impl Flip {
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_scale = min(1000 / height, 1500 / width);
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

        Flip {
            width,
            height,
            pixel_scale,
            window,
            pixels,
            sim,
            event_loop,
            palette: (0.000000000001, 0.0),
        }
    }

    pub fn run(mut self) {
        let mut cursor_position: Option<(f64, f64)> = None;
        let mut last_frame_start = Instant::now();
        let mut frame_time = Duration::ZERO;
        let mut ticker: u8 = 0;
        let mut step_time: f64 = 0.0;

        self.event_loop.run(move |event, _, control_flow| {
            ticker = ticker.wrapping_add(8);
            if ticker == 0 {
                self.window.set_title(&format!(
                    "Eulerian Fluid Simulation - FPS: {:.0}",
                    1.0 / frame_time.as_secs_f64() as f64
                ));
            }

            if TIME_DETAILS && ticker == 0 {
                println!(
                    "Time for render: {}",
                    last_frame_start.elapsed().as_secs_f64() - step_time
                );
            }
            if DRAW_PRESSURE {
                display(
                    self.pixels.frame_mut(),
                    self.sim.get_pressure(),
                    self.sim.get_s(),
                    &mut self.palette,
                    ticker == 0,
                );
            } else if DRAW_U {
                display(
                    self.pixels.frame_mut(),
                    &self.sim.get_u(),
                    self.sim.get_s(),
                    &mut self.palette,
                    ticker == 0,
                );
            } else {
                display(
                    self.pixels.frame_mut(),
                    &self.sim.get_smoke(),
                    self.sim.get_s(),
                    &mut self.palette,
                    ticker == 0,
                );
            }

            _ = self.pixels.render();

            frame_time = last_frame_start.elapsed();
            last_frame_start = Instant::now();

            // self.sim.step(frame_time.as_secs_f64());
            self.sim.step(1.0 / 12.0);
            if TIME_DETAILS && ticker == 0 {
                step_time = last_frame_start.elapsed().as_secs_f64();
                println!("Time for simulation step: {}", step_time);
            }
            // *control_flow =
            //     ControlFlow::WaitUntil(last_frame_start + Duration::from_millis(1000000));
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
                                self.sim.print_info(grid_x as usize, grid_y as usize);
                            } else {
                                let radius = (self.width / 9) as f32;
                                println!("Circle drawn");
                                self.sim.draw_filled_circle(grid_x, grid_y, radius);
                            }
                        }
                    }

                    WindowEvent::MouseInput {
                        state: winit::event::ElementState::Pressed,
                        button: winit::event::MouseButton::Right,
                        ..
                    } => {
                        self.sim.reset_walls();
                        self.sim.reset_velocities();
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        cursor_position = Some((position.x, position.y));
                    }
                    _ => {}
                },
                _ => {}
            }
        });
    }
}

fn update_palette(imag: &Vec<f64>, palette: &mut (f64, f64)) {
    let min = imag.iter().fold(f64::MAX, |acc, &x| acc.min(x));
    let max = imag.iter().fold(f64::MIN, |acc, &x| acc.max(x));
    *palette = ((max - min).max(0.000001), min);
}

fn display(
    frame: &mut [u8],
    imag: &Vec<f64>,
    mask: &Vec<f64>,
    palette: &mut (f64, f64),
    update: bool,
) {
    //Go from [0.0, 1.0] to [0,255]; then assign that value to R, G, B and make A=255.
    // dbg!(frame.len());
    if update {
        update_palette(imag, palette);
    }

    let buffer: Vec<u8> = imag
        .iter()
        .map(|x| (x - palette.1) / (palette.0))
        .zip(mask.iter())
        .map(|(y, &fluid)| {
            if RGB {
                hsv_to_rgb(
                    (if y.is_nan() { 1.0 } else { y.clamp(0.0, 1.0) }) * 300.0,
                    (palette.0).tanh().clamp(0.5, 1.0),
                    fluid,
                )
            } else {
                (
                    ((if y.is_nan() { 1.0 } else { y.clamp(0.0, 1.0) }) * fluid * 255.0) as u8,
                    ((if y.is_nan() { 1.0 } else { y.clamp(0.0, 1.0) }) * fluid * 255.0) as u8,
                    ((if y.is_nan() { 1.0 } else { y.clamp(0.0, 1.0) }) * fluid * 255.0) as u8,
                )
            }
        })
        .map(|(r, g, b)| [r as u8, g as u8, b as u8, 255])
        .flatten()
        .collect();
    frame.copy_from_slice(&buffer);
}
