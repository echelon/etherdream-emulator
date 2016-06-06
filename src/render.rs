// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

extern crate rand;

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
use glium_graphics::{                                                           
    Flip, Glium2d, GliumWindow, OpenGL, Texture, TextureSettings
};

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

/*impl AtomicPointBuffer {
  pub fn new() -> AtomicPointBuffer {
    let buffer = PointBuffer::new();
    AtomicPointBuffer {
      holder: Arc::new(RwLock::new(buffer))
    }
  }

  pub get(&self) -> &mut PointBuffer {
  }
}*/

pub fn gl_window(buffer: Arc<RwLock<PointBuffer>>) {
  let opengl = OpenGL::V3_2;
  let (w, h) = (1280, 960);
  let ref mut window: GliumWindow =
    WindowSettings::new("glium_graphics: image_test", [w, h])
    .exit_on_esc(true).opengl(opengl).build().unwrap();


  let mut g2d = Glium2d::new(opengl, window); 
  while let Some(e) = window.next() { 
    if let Some(args) = e.render_args() { 
      use graphics::*;

      let mut target = window.draw();
      g2d.draw(&mut target, args.viewport(), |c, g| {

        let point_transform = c.transform.scale(0.05, 0.05);
        let mut rng = rand::thread_rng();

        clear([1.0; 4], g);

        // Background
        Rectangle::new([0.0, 0.0, 0.0, 1.0])
          .draw([0.0, 0.0, 1280.0, 1280.0], &c.draw_state, c.transform, g);

        match (*buffer).read() {
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
        }

        sleep(Duration::from_millis(50)); 
      });

      target.finish().unwrap();
    }
  }
}

pub fn map_x(x: u16, width: u16) -> f64 {
  let scale = width as f64 / 65535.0;
  x as f64 * scale
}

pub fn map_y(y: u16, height: u16) -> f64 {
  let scale = height as f64 / 65535.0;
  y as f64 * scale
}

