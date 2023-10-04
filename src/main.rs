use std::cmp::min;
use std::time::Duration;
use num_complex::Complex;
use bitvec::prelude::*;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color,PixelFormatEnum};
use sdl2::gfx::primitives::DrawRenderer;

const WIDTH: usize = 1024;
const HEIGHT: usize = 768;
const SIZE: usize = WIDTH*HEIGHT;
const RADIUS: f64 = 1000.0;

struct Grid {
    step: usize,
    c: Vec<Complex<f64>>,
    z: Vec<Complex<f64>>,
    diverged: BitVec,
    pixels: Vec<u8>,
}

impl Grid {
    fn new(c1: Complex<f64>, step: f64) -> Grid {
        let zero = Complex{re: 0.0, im: 0.0};
        let mut c = vec![zero; SIZE];

        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                c[i*WIDTH+j] = c1 + Complex{
                    re: j as f64 * step,
                    im: i as f64 * step,
                };
            };
        }
        Grid {
            step: 0,
            c: c,
            z: vec![zero; SIZE],
            diverged: bitvec![0; SIZE],
            pixels: vec![0; 4*SIZE],
        }
    }
    fn update(&mut self) {
        for i in 0..SIZE {
            if self.diverged[i] {
                continue
            };
            for step in self.step..self.step+20 {
                self.z[i] = self.z[i] * self.z[i] + self.c[i];
                let n2 = self.z[i].norm_sqr();
                if n2 > RADIUS {
                    self.diverged.set(i, true);
                    let step = min(self.step + step, 20) as f64 - n2.ln().ln();
                    let val = 120u8 + (128.0 * step / 20.0) as u8;
                    for j in 0..3 {
                        self.pixels[4*i+j] = val;
                    }
                    break;
                }
            }
        }
        self.step += 20;
    }
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video = sdl_context.video()?;
    let mut event_pump = sdl_context.event_pump()?;

    let mut canvas = video
        .window("Mandelbrot", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;

    let mut grid = Grid::new(Complex{re: -3.0, im: -1.5}, 4.0/WIDTH as f64);

    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_streaming(
        PixelFormatEnum::RGB888,
        WIDTH as u32,
        HEIGHT as u32,
    ).map_err(|e| e.to_string())?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}
                | Event::KeyDown { keycode: Some(Keycode::Escape | Keycode::Return), .. }
                => {
                    break 'running
                },
                _ => continue,
            }
        }
        grid.update();
        texture.update(None, &grid.pixels, 4*WIDTH).map_err(|e| e.to_string())?;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    };

    Ok(())
}
