use std::cmp::{min,max};
use std::time::Duration;
use std::thread;

use num_complex::Complex;
use bitvec::prelude::*;
use crossbeam::channel::{unbounded,Receiver,Sender};

use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color,PixelFormatEnum};
use sdl2::gfx::primitives::DrawRenderer;

const STEPS: usize = 20;
const WIDTH: usize = 1024;
const HEIGHT: usize = 768;
const RADIUS: f64 = 1000.0;
const NTHREADS: usize = 8;
const ROWS: usize = HEIGHT/NTHREADS;
const SIZE: usize = WIDTH*ROWS;

struct Grid {
    idx: usize,
    step: usize,
    eps: f64,
    c: Vec<Complex<f64>>,
    z: Vec<Complex<f64>>,
    diverged: BitVec,
    pixels: Vec<u8>,
    zoom_hist: Vec<(f64, Complex<f64>)>,
}

impl Grid {
    fn new(idx: usize, c: Complex<f64>, eps: f64) -> Grid {
        let zero = Complex::new(0.0, 0.0);
        let mut grid = Grid {
            idx: idx,
            step: 0,
            eps: eps,
            c: vec![zero; SIZE],
            z: vec![zero; SIZE],
            diverged: bitvec![0; SIZE],
            pixels: vec![0; 4*SIZE],
            zoom_hist: Vec::new(),
        };
        grid.init(c + Complex::new(0.0, eps*(idx*ROWS) as f64));
        grid
    }

    fn init(&mut self, c: Complex<f64>) {
        self.step = 0;
        for i in 0..ROWS {
            for j in 0..WIDTH {
                self.c[i*WIDTH+j] = c + Complex{
                    re: j as f64 * self.eps,
                    im: i as f64 * self.eps,
                };
            };
        };
        for i in 0..SIZE {
            self.z[i] = Complex::new(0.0, 0.0);
            self.diverged.set(i, false);
        };
        for i in 0..4*SIZE {
            self.pixels[i] = 0;
        }
        println!("{} {}", self.eps, self.c[1].re - self.c[0].re);
    }

    fn zoom_in(&mut self, corner: (usize, usize)) {
        self.zoom_hist.push((self.eps, self.c[0]));
        self.eps /= 2.0;
        self.init(self.c[corner.1 * WIDTH + corner.0]);
    }

    fn zoom_out(&mut self) {
        let (eps, c) = match self.zoom_hist.pop() {
            None => return,
            Some(x) => x,
        };
        self.eps = eps;
        self.init(c);
    }

    fn update(&mut self) {
        for i in 0..SIZE {
            if self.diverged[i] {
                continue
            };
            for step in self.step..self.step+STEPS {
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
        self.step += STEPS;
    }
}

fn work(rcv: Receiver<Option<Grid>>, snd: Sender<Grid>) {
    loop {
        let mut grid = match rcv.recv() {
            Ok(x) => match x {
                Some(grid) => grid,
                None => break,
            },
            Err(_) => break,
        };
        grid.update();
        match snd.send(grid) {
            Ok(_) => (),
            Err(_) => break,
        };
    };
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

    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_streaming(
        PixelFormatEnum::RGB888,
        WIDTH as u32,
        HEIGHT as u32,
    ).map_err(|e| e.to_string())?;

    let (snd_main, rcv_thrd) = unbounded();
    let (snd_thrd, rcv_main) = unbounded();
    for _ in 1..NTHREADS {
        let rcv = rcv_thrd.clone();
        let snd = snd_thrd.clone();
        thread::spawn(move || {work(rcv, snd)});
    }
    thread::spawn(move || { work(rcv_thrd, snd_thrd); });

    let mut grids = Vec::new();
    for i in 0..NTHREADS {
        grids.push(Grid::new(
            i,
            Complex::new(-3.0, -1.5),
            4.0/WIDTH as f64,
        ));
    };

    let mut corner = (0usize, 0usize);
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}
                | Event::KeyDown { keycode: Some(Keycode::Escape | Keycode::Return), .. }
                => {
                    break 'running
                },
                Event::MouseMotion { x, y, .. } => {
                    let w = WIDTH as i32;
                    let h = HEIGHT as i32;
                    corner.0 = max(0, min(x - w / 4, w / 2)) as usize /16*16;
                    corner.1 = max(0, min(y - h / 4, h / 2)) as usize /16*16;
                },
                //Event::MouseButtonDown {..} => {
                //    grid.zoom_in(corner);
                //},
                //Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                //    grid.zoom_out();
                //}
                _ => continue,
            }
        }
        while let Ok(grid) = rcv_main.try_recv() {
            texture.update(
                Rect::new(0, (grid.idx*ROWS) as i32, WIDTH as u32, ROWS as u32),
                &grid.pixels,
                4*WIDTH,
            ).map_err(|e| e.to_string())?;
            grids.push(grid);
        };
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        if grids.len() == NTHREADS {
            canvas.copy(&texture, None, None)?;
            while let Some(grid) = grids.pop() {
                snd_main.send(Some(grid)).map_err(|e| e.to_string())?;
            };
        }
        canvas.rectangle(
            corner.0 as i16,
            corner.1 as i16,
            (corner.0 + WIDTH / 2) as i16,
            (corner.1 + HEIGHT / 2) as i16,
            Color::RGB(255, 0, 0)
        )?;
        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    };

    Ok(())
}
