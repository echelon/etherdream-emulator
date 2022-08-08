// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use glium_graphics::Glium2d;
use glium_graphics::GliumWindow;
use glium_graphics::OpenGL;
use graphics::*;
use ilda::limit;
use pipeline::Pipeline;
use piston::input::*;
use piston::window::WindowSettings;
use std::process;
use std::sync::Arc;
use RuntimeOpts;

/// Initial window dimensions.
const INITIAL_WINDOW_DIMENSIONS: [u32; 2] = [600, 600];

/// RGBA window background color.
/// Not completely black so that laser blanking can be seen.
const BG_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

pub fn gl_window(pipeline: Arc<Pipeline>, runtime_opts: &RuntimeOpts) {
    let opengl = OpenGL::V3_2;
    let ref mut window: GliumWindow =
        WindowSettings::new("EtherDream Emulator", INITIAL_WINDOW_DIMENSIONS)
            .exit_on_esc(true)
            .build()
            .unwrap();

    let mut g2d = Glium2d::new(opengl, window);
    while let Some(e) = window.next() {
        if let Some(args) = e.render_args() {
            let mut frame = window.draw();
            g2d.draw(&mut frame, args.viewport(), |ctx, gfx| {
                // Draw background color
                Rectangle::new(BG_COLOR).draw(
                    [0.0, 0.0, args.window_size[0], args.window_size[1]],
                    &ctx.draw_state,
                    ctx.transform,
                    gfx,
                );

                let result = pipeline.dequeue(1_000);
                let points = match result {
                    Err(_) => Vec::new(), // TODO
                    Ok(points) => points,
                };

                for point in points {
                    let x = map_x(point.x, args.window_size[0] as u32);
                    let y = map_y(point.y, args.window_size[1] as u32);
                    let r = map_color(point.r);
                    let g = map_color(point.g);
                    let b = map_color(point.b);

                    Ellipse::new([r, g, b, 1.0]).draw(
                        [
                            // Position
                            x,
                            y,
                            // Size of shape.
                            runtime_opts.point_size,
                            runtime_opts.point_size,
                        ],
                        &ctx.draw_state,
                        ctx.transform,
                        gfx,
                    );
                }
            });

            frame.finish().unwrap();
        }
    }

    println!("Window closed. Terminating process.");
    process::exit(0);
}

// FIXME: This is abhorrent.
#[inline]
pub fn map_x(x: i16, width: u32) -> f64 {
    let tx = (x as i32).saturating_add(limit::MAX_X as i32);
    let scale = width as f64 / limit::WIDTH as f64;
    tx as f64 * scale
}

// FIXME: This is abhorrent.
#[inline]
pub fn map_y(y: i16, height: u32) -> f64 {
    // NB: Have to invert y since the vertical coordinate system transforms.
    let ty = ((y * -1) as i32).saturating_add(limit::MAX_Y as i32);
    let scale = height as f64 / limit::HEIGHT as f64;
    ty as f64 * scale
}

/// Convert color space from ILDA to float.
#[inline]
pub fn map_color(c: u16) -> f32 {
    c as f32 / 65535.0
}