// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

extern crate rand;

use std::process;
use dac::Dac;
use glium::DisplayBuild;
use graphics::draw_state::Blend;
use piston::input::*;
use piston::window::WindowSettings;
use protocol::Point;
use rand::Rng;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use ilda::limit;
use glium_graphics::{
    Flip, Glium2d, GliumWindow, OpenGL, Texture, TextureSettings
};

//const WINDOW_WIDTH : u32 = 1280;
//const WINDOW_HEIGHT : u32 = 1280;

const WINDOW_WIDTH : u32 = 600;
const WINDOW_HEIGHT : u32 = 600;

pub struct TimedPoint {
  pub point: Point,
  pub instant: Instant,
}

impl TimedPoint {
  pub fn new(point: Point) -> TimedPoint {
    TimedPoint {
      point: point,
      instant: Instant::now(),
    }
  }

  pub fn can_draw(&self) -> bool {
    self.instant.elapsed() < Duration::from_millis(100)
  }
}

pub struct PointBuffer {
  buffer: Vec<TimedPoint>,
  next: usize,
  capacity: usize,
}

#[derive(Clone)]
pub struct AtomicPointBuffer {
  holder: Arc<RwLock<PointBuffer>>,
}

impl PointBuffer {
  pub fn new() -> PointBuffer {
    PointBuffer {
      buffer: Vec::with_capacity(200),
      next: 0,
      capacity: 200,
    }
  }

  pub fn add(&mut self, point: TimedPoint) {
    self.buffer.insert(self.next, point);
    self.next = (self.next + 1) % self.capacity; // FIXME: Capacity
  }
  pub fn read(&self) -> &Vec<TimedPoint> {
    &self.buffer
  }
}

pub fn gl_window(dac: Arc<Dac>) {
  let opengl = OpenGL::V3_2;
  let ref mut window: GliumWindow =
    WindowSettings::new("glium_graphics: image_test", [WINDOW_WIDTH, WINDOW_HEIGHT])
    .exit_on_esc(true).opengl(opengl).build().unwrap();


  let mut g2d = Glium2d::new(opengl, window);
  while let Some(e) = window.next() {
    if let Some(args) = e.render_args() {
      use graphics::*;

      /*let point_ring_buffer = Vec::new();
      let ring_size : usize = 1000;
      let ring_pos : usize = 0;*/

      let mut target = window.draw();
      g2d.draw(&mut target, args.viewport(), |ctx, gfx| {

        let point_transform = ctx.transform.scale(0.05, 0.05);
        let mut rng = rand::thread_rng();

        clear([1.0; 4], gfx);

        // Background
        Rectangle::new([0.2, 0.2, 0.2, 1.0])
          .draw([0.0, 0.0, WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64],
                &ctx.draw_state,
                ctx.transform,
                gfx);

        let points = dac.drain_points();
        //println!("points len: {}", points.len());

        let mut i = 0;
        for point in points {
          i += 1;
          // TODO: This is a lame hack to deal with queue consumption being too slow
          if i % 50 != 0 {
            //continue;
          }

          let x = map_x(point.x, WINDOW_WIDTH);
          let y = map_y(point.y, WINDOW_HEIGHT);

          //println!("{}, {}", point.x, point.y);
          //println!("{}, {}", x, y);

          let r = map_color(point.r);
          let gr = map_color(point.g);
          let b = map_color(point.b);

          Ellipse::new([r, gr, b, 1.0])
            .draw([
                  // Position
                  x,
                  y,
                  // Size of shape.
                  10.0,
                  10.0,
            ],
            &ctx.draw_state, ctx.transform, gfx);
        }
        /*match (*buffer).read() {
          Err(_) => {},
          Ok(pb) => {
            let points = pb.read();

            for timed_point in points {
              if !timed_point.can_draw() {
                continue;
              }

              let x = map_x(timed_point.point.x, 1280);
              let y = map_y(timed_point.point.y, 960);

              println!("{}, {}", x, y);
              println!("{}, {}", timed_point.point.x, timed_point.point.y);

              let r = rng.gen_range(0.0, 1.0);
              let gr = rng.gen_range(0.0, 1.0);
              let b = rng.gen_range(0.0, 1.0);

              Ellipse::new([r, gr, b, 1.0])
                .draw([
                      // Position
                      x,
                      y,
                      // Size of shape.
                      10.0,
                      10.0,
                ],
                &c.draw_state, c.transform, g);

            }
          },
        }*/

        sleep(Duration::from_millis(50));
      });

      target.finish().unwrap();
    }
  }

  println!("Terminating process.");
  process::exit(0);
}

// FIXME: This is abhorrent.
pub fn map_x(x: i16, width: u32) -> f64 {
  let tx = (x as i32).saturating_add(limit::MAX_X as i32);
  let scale = width as f64 / limit::WIDTH as f64;
  tx as f64 * scale
}

// FIXME: This is abhorrent.
pub fn map_y(y: i16, height: u32) -> f64 {
  // NB: Have to invert y since the vertical coordinate system transforms.
  let ty = ((y * -1) as i32).saturating_add(limit::MAX_Y as i32);
  let scale = height as f64 / limit::HEIGHT as f64;
  ty as f64 * scale
}

pub fn map_color(c: u16) -> f32 {
  c as f32 / 65535.0
}

/*/// Transform x-coordinate.
fn t_x(x : i16, img_width: u32) -> u32 {
  // FIXME: This is abhorrent.
  let ix = (x as i32).saturating_add(limit::MAX_X as i32);
  let scale = (img_width as f64) / (limit::WIDTH as f64);
  ((ix as f64 * scale) as i32).abs() as u32
}

/// Transform y-coordinate.
fn t_y(y : i16, img_height: u32) -> u32 {
  // FIXME: This is abhorrent.
  // NB: Have to invert y since the vertical coordinate system transforms.
  let iy = ((y * -1) as i32).saturating_add(limit::MAX_Y as i32);
  let scale = (img_height as f64) / (limit::HEIGHT as f64);
  ((iy as f64 * scale) as i32).abs() as u32
}
*/

