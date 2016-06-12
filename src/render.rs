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

const INITIAL_WINDOW_WIDTH : u32 = 600;
const INITIAL_WINDOW_HEIGHT : u32 = 600;

/// RGBA window background color.
/// Not completely black so that laser blanking can be seen.
const BG_COLOR : [f32; 4] = [0.1, 0.1, 0.1, 1.0];

pub fn gl_window(dac: Arc<Dac>) {
  let opengl = OpenGL::V3_2;
  let ref mut window: GliumWindow =
    WindowSettings::new("EtherDream Emulator", 
                        [INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT])
      .exit_on_esc(true)
      .opengl(opengl)
      .build()
      .unwrap();

  let mut g2d = Glium2d::new(opengl, window);
  while let Some(e) = window.next() {
    if let Some(args) = e.render_args() {
      use graphics::*;

      let mut target = window.draw();
      g2d.draw(&mut target, args.viewport(), |ctx, gfx| {

        let point_transform = ctx.transform.scale(0.05, 0.05);
        let mut rng = rand::thread_rng();

        clear([1.0; 4], gfx);

        // Draw background color
        Rectangle::new(BG_COLOR)
          .draw([0.0, 0.0, args.width as f64, args.height as f64],
                &ctx.draw_state,
                ctx.transform,
                gfx);

        for point in dac.drain_points() {
          let x = map_x(point.x, args.width);
          let y = map_y(point.y, args.height);
          let r = map_color(point.r);
          let g = map_color(point.g);
          let b = map_color(point.b);

          Ellipse::new([r, g, b, 1.0])
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
      });

      target.finish().unwrap();
    }
  }

  println!("Window closed. Terminating process.");
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

/// Convert color space from ILDA to float.
pub fn map_color(c: u16) -> f32 {
  c as f32 / 65535.0
}

