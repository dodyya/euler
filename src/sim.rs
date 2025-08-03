use crate::util::Array2D;
use paste::paste;
use std::cmp::{max, min};
#[derive(Debug)]
pub struct Simulation {
    width: usize,
    height: usize,
    u: Array2D<f64>, // x; lie on vertical cell boundaries
    v: Array2D<f64>, // y; lie on horizontal cell boundaries,
    s: Array2D<f64>,
    p: Array2D<f64>,
    smoke: Array2D<f64>,
}

const OVERRELAXATION_FACTOR: f64 = 1.9;
const NUM_PROJ_ITERATIONS: u32 = 40;
const WITH_GRAVITY: bool = false;
const GRAVITY: f64 = 7.2;
const DENSITY: f64 = 10.0;
const WINDSPEED: f64 = 30.0;

const H: f64 = 1.0;
const DRAW_OBSTACLE: bool = true;

macro_rules! create_sample_method {
    ($field:ident,$dx:expr, $dy:expr ) => {
        paste! {
            fn [<sample_ $field>](&self, x_in: f64, y_in: f64) -> f64 {
                let x = H.max(x_in.min(self.width as f64 * H));
                let y = H.max(y_in.min(self.height as f64 * H));

                let x0 = min(((x - $dx) / H).floor() as usize, self.width - 1);
                let tx = ((x - $dx) - x0 as f64 * H) / H;
                let x1 = min(x0 + 1, self.width - 1);

                let y0 = min(((y - $dy) / H).floor() as usize, self.height - 1);
                let ty = ((y - $dy) - y0 as f64 * H) / H;
                let y1 = min(y0 + 1, self.height - 1);

                let sx = 1.0 - tx;
                let sy = 1.0 - ty;

                return sx * sy * self.$field[(x0, y0)]
                    + tx * sy * self.$field[(x1, y0)]
                    + tx * ty * self.$field[(x1, y1)]
                    + sx * ty * self.$field[(x0, y1)];
            }
        }
    };
}

impl Simulation {
    pub fn new(width: usize, height: usize) -> Self {
        let mut u = Array2D::new(width + 1, height);
        let mut s = Array2D::fill(1.0, width, height);
        for y in 0..height {
            u[(0, y)] = WINDSPEED;
            u[(width, y)] = WINDSPEED;
        }

        if DRAW_OBSTACLE {
            draw_filled_circle(
                &mut s,
                width as i32 / 3 + 5,
                height as i32 / 2 - 1,
                width as f32 / 7.0,
                0.0,
            );
        }

        let mut smoke = Array2D::new(width, height);
        let h2 = 5;
        let center = height / 2;
        for y in center - h2..=center + h2 {
            smoke[(0, y)] = 1.0;
        }

        Simulation {
            width,
            height,
            u,
            v: Array2D::new(width, height + 1),
            s,
            p: Array2D::new(width, height),
            smoke,
        }
    }

    fn div(&self, x: usize, y: usize) -> f64 {
        OVERRELAXATION_FACTOR
            * (self.u[(x + 1, y)] - self.u[(x, y)] + self.v[(x, y + 1)] - self.v[(x, y)])
    }

    pub fn gravity(&mut self, dt: f64) {
        for y in 1..self.height {
            for x in 0..self.width {
                if self.open_v(x, y) {
                    self.v[(x, y)] += GRAVITY * dt;
                }
            }
        }
    }

    fn projection(&mut self, dt: f64) {
        self.p.zero();
        for _ in 0..NUM_PROJ_ITERATIONS as usize {
            for y in 0..self.height {
                for x in 0..self.width {
                    if self.s[(x, y)] == 0.0 {
                        continue;
                    }
                    let d = self.div(x, y);
                    let s1 = self.s(x as i32 - 1, y as i32);
                    let s2 = self.s(x as i32 + 1, y as i32);
                    let s3 = self.s(x as i32, y as i32 - 1);
                    let s4 = self.s(x as i32, y as i32 + 1);
                    let s = s1 + s2 + s3 + s4;
                    self.u[(x, y)] += d * s1 / s;
                    self.u[(x + 1, y)] -= d * s2 / s;
                    self.v[(x, y)] += d * s3 / s;
                    self.v[(x, y + 1)] -= d * s4 / s;

                    self.p[(x, y)] += d / s * DENSITY * H / dt;
                }
            }
        }
    }

    fn s(&self, x: i32, y: i32) -> f64 {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            self.s[(x as usize, y as usize)]
        } else if x == -1 {
            0.0
        } else if x == self.width as i32 {
            0.0
        } else if y == -1 || y == self.height as i32 {
            0.0
        } else {
            panic!(
                "({},{}) not in [-1, {}]x[-1, {}]",
                x, y, self.width, self.height
            );
        }
    }

    fn open_u(&self, x: usize, y: usize) -> bool {
        self.s(x as i32, y as i32) != 0.0 && self.s(x as i32 - 1, y as i32) != 0.0
    }
    fn open_v(&self, x: usize, y: usize) -> bool {
        self.s(x as i32, y as i32) != 0.0 && self.s(x as i32, y as i32 - 1) != 0.0
    }

    fn avg_u(&self, x: usize, y: usize) -> f64 {
        (self.u[(x, y - 1)] + self.u[(x, y)] + self.u[(x + 1, y - 1)] + self.u[(x + 1, y)]) * 0.25
    }
    fn avg_v(&self, x: usize, y: usize) -> f64 {
        // dbg!(x, y);
        (self.v[(x - 1, y)] + self.v[(x, y)] + self.v[(x - 1, y + 1)] + self.v[(x, y + 1)]) * 0.25
    }

    create_sample_method!(u, 0.0, H / 2.0);
    create_sample_method!(v, H / 2.0, 0.0);
    create_sample_method!(smoke, H / 2.0, H / 2.0);
    pub fn step(&mut self, dt: f64) {
        if WITH_GRAVITY {
            self.gravity(dt);
        }
        self.projection(dt);
        self.advection(dt);
        self.smoke_advection(dt);
    }

    fn advection(&mut self, dt: f64) {
        let mut new_u = self.u.clone();
        let mut new_v = self.v.clone();
        for j in 0..=self.height {
            for i in 0..=self.width {
                if i > 0 && i < self.height && self.open_u(i, j) {
                    let mut x = i as f64 * H;
                    let mut y = j as f64 * H + 0.5 * H;
                    let u = new_u[(i, j)];
                    let v = self.avg_v(i, j);

                    x -= dt * u;
                    y -= dt * v;

                    new_u[(i, j)] = self.sample_u(x, y);
                    // println!("U hit ({},{})", i, j);
                }

                if j > 0 && j < self.width && self.open_v(i, j) {
                    let mut x = i as f64 * H + 0.5 * H;
                    let mut y = j as f64 * H;
                    let v = new_v[(i, j)];
                    let u = self.avg_u(i, j);

                    x -= dt * u;
                    y -= dt * v;

                    new_v[(i, j)] = self.sample_v(x, y);
                }
            }
        }
        self.u = new_u;
        self.v = new_v;
    }

    fn smoke_advection(&mut self, dt: f64) {
        let mut new_smoke = self.smoke.clone();
        for j in 1..self.height - 1 {
            for i in 1..self.width {
                if self.s[(i, j)] != 1.0 {
                    continue;
                }

                let u = 0.5 * (self.u[(i, j)] + self.u[(i + 1, j)]);
                let v = 0.5 * (self.v[(i, j)] + self.v[(i, j + 1)]);

                let x = i as f64 * H + 0.5 * H - dt * u;
                let y = j as f64 * H + 0.5 * H - dt * v;

                new_smoke[(i, j)] = self.sample_smoke(x, y);
            }
        }
        self.smoke = new_smoke;
    }

    pub fn print_info(&self, x: usize, y: usize) {
        println!("(x,y) = ({},{}):", x, y);
        println!("u-flow: {:.5}", 0.5 * (self.u[(x, y)] + self.u[(x + 1, y)]));
        println!("v-flow: {:.5}", 0.5 * (self.v[(x, y)] + self.v[(x, y + 1)]));
        println!("s: {:.5}", self.s[(x, y)]);
        println!("p: {:.5}", self.p[(x, y)]);
        println!("smoke: {:.5}", self.smoke[(x, y)]);
    }

    pub fn get_speed(&self) -> Vec<f64> {
        let mut speed = Vec::new();
        for j in 0..self.height {
            for i in 0..self.width {
                let u = 0.5 * (self.u[(i, j)] + self.u[(i + 1, j)]);
                let v = 0.5 * (self.v[(i, j)] + self.v[(i, j + 1)]);
                speed.push((u * u + v * v).sqrt());
                // speed.push(u);
            }
        }
        speed
    }
    pub fn get_u(&self) -> Vec<f64> {
        let mut speed = Vec::new();
        for j in 0..self.height {
            for i in 0..self.width {
                let u = 0.5 * (self.u[(i, j)] + self.u[(i + 1, j)]);
                speed.push(u);
            }
        }
        speed
    }

    pub fn get_smoke(&self) -> &Vec<f64> {
        &self.smoke.data
    }

    pub fn get_pressure(&self) -> &Vec<f64> {
        &self.p.data
    }

    pub fn get_s(&self) -> &Vec<f64> {
        &self.s.data
    }

    pub fn draw_filled_circle(&mut self, center_x: i32, center_y: i32, radius: f32) {
        draw_filled_circle(&mut self.s, center_x, center_y, radius, 0.0);
    }

    pub fn reset_walls(&mut self) {
        self.s.reset(1.0);
    }

    pub fn reset_velocities(&mut self) {
        *self = Self::new(self.width, self.height);
    }
}

fn draw_filled_circle(
    pixels: &mut Array2D<f64>,
    center_x: i32,
    center_y: i32,
    radius: f32,
    value: f64,
) {
    let r_squared = (radius * radius) as i32;

    for y in (center_y - radius.ceil() as i32)..=(center_y + radius.ceil() as i32) {
        for x in (center_x - radius.ceil() as i32)..=(center_x + radius.ceil() as i32) {
            // Check bounds
            if x >= 0 && y >= 0 && x < pixels.width as i32 && y < pixels.height as i32 {
                let dx = x - center_x;
                let dy = y - center_y;

                if dx * dx + dy * dy <= r_squared {
                    pixels[(x as usize, y as usize)] = value;
                }
            }
        }
    }
}
