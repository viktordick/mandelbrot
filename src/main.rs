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
    anchor: Complex<f64>,
    eps: f64,
    step: usize,
    c: Vec<Complex<f64>>,
    z: Vec<Complex<f64>>,
    diverged: BitVec,
    pixels: Vec<u8>,
}

impl Grid {
    fn new(idx: usize, anchor: Complex<f64>, eps: f64) -> Grid {
        let zero = Complex::new(0.0, 0.0);
        Grid {
            idx: idx,
            anchor: anchor,
            step: 0,
            eps: eps,
            c: vec![zero; SIZE],
            z: vec![zero; SIZE],
            diverged: bitvec![0; SIZE],
            pixels: vec![0; 4*SIZE],
        }
    }

    fn init(&mut self) {
        let c = self.anchor + Complex::new(0.0, self.eps*(self.idx * ROWS) as f64);
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

enum Task {
    Update(Grid),
    Init(Grid),
    Terminate,
}

fn work(rcv: Receiver<Task>, snd: Sender<Grid>) {
    loop {
        let task = match rcv.recv() {
            Ok(task) => task,
            Err(_) => break,
        };
        let grid = match task {
            Task::Terminate => break,
            Task::Update(mut grid) => {grid.update(); grid},
            Task::Init(mut grid) => {grid.init(); grid},
        };
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

    let anchor = Complex::new(-3.0, -1.5);
    let eps = 4.0/WIDTH as f64;
    for i in 0..NTHREADS {
        snd_main.send(Task::Init(Grid::new(i, anchor, eps))).map_err(|e| e.to_string())?;
    };

    let mut zoom_hist = Vec::new();
    let mut zoom = (anchor, eps);

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
                Event::MouseButtonDown {..} => {
                    // Zoom in
                    zoom_hist.push(zoom);
                    let eps = zoom.1;
                    let anchor = zoom.0 + Complex::new(corner.0 as f64, corner.1 as f64) * eps;
                    zoom = (anchor, eps/2.0);
                },
                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                    // Zoom out
                    if let Some(x) = zoom_hist.pop() {
                        zoom = x;
                    };
                }
                _ => continue,
            }
        }
        let mut received = 0;
        while received < NTHREADS{
            let mut grid = match rcv_main.try_recv() {
                Err(_) => break,
                Ok(grid) => grid,
            };
            received += 1;
            texture.update(
                Rect::new(0, (grid.idx*ROWS) as i32, WIDTH as u32, ROWS as u32),
                &grid.pixels,
                4*WIDTH,
            ).map_err(|e| e.to_string())?;
            let task = if (grid.anchor, grid.eps) != zoom {
                grid.anchor = zoom.0;
                grid.eps = zoom.1;
                Task::Init(grid)
            } else {
                Task::Update(grid)
            };
            snd_main.send(task).map_err(|e| e.to_string())?;
        };
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.copy(&texture, None, None)?;
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
    for _ in 0..NTHREADS {
        snd_main.send(Task::Terminate).unwrap();
    };

    Ok(())
}
