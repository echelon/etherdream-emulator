// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use dac::DacFrame;
use ilda::limit;
use byteorder::LittleEndian;
use std::thread;
use std::time::Duration;
use byteorder::ReadBytesExt;
use protocol::Point;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::io::Cursor;
use error::EmulatorError;

pub struct DrawPoint {
  pub x: f64,
  pub y: f64,
  pub r: f32,
  pub g: f32,
  pub b: f32,
}

/// A separate thread to consume raw points off the wire and translate them into
/// graphical points ready to render. This takes load off the DAC thread as well
/// as the OpenGL/drawing thread.
pub struct Pipeline {
  input: Mutex<VecDeque<DacFrame>>,
  output: Mutex<VecDeque<DrawPoint>>,
  frame_limit: usize,
  point_limit: usize,
}

impl Pipeline {
  /// CTOR.
  pub fn new() -> Pipeline {
    Pipeline {
      input: Mutex::new(VecDeque::new()),
      output: Mutex::new(VecDeque::new()),
      frame_limit: 10_000,
      point_limit: 100_000,
    }
  }

  pub fn input_len(&self) -> Result<usize, EmulatorError> {
    let mut lock = self.output.lock()?;
    let len = (*lock).len();
    Ok(len)
  }

  /// Enqueue frames from the network thread.
  pub fn enqueue(&self, frame: DacFrame) -> Result<(), EmulatorError> {
    let mut lock = self.input.lock()?;
    if (*lock).len() > self.frame_limit {
      return Err(EmulatorError::PipelineFull);
    }
    (*lock).push_back(frame);
    Ok(())
  }

  /// Dequeue points from the graphics thread.
  pub fn dequeue(&self, num_points: usize)
                 -> Result<Vec<DrawPoint>, EmulatorError> {
    let mut buf = Vec::new();
    let mut lock = self.output.lock()?;

    while buf.len() < num_points {
      match (*lock).pop_front() {
        None => return Ok(buf), // Return fewer frames than asked for.
        Some(frame) => buf.push(frame),
      }
    }
    (*lock) = VecDeque::new();
    Ok(buf)
  }

  /// Run by a separate thread from network and graphics.
  pub fn process(&self) -> ! {
    //thread::sleep(Duration::from_secs(1000));

    loop {
      {
        let mut lock = self.input.lock().unwrap(); // Fatal error.
        let input_len = (*lock).len();

        let mut lock = self.output.lock().unwrap(); // Fatal error.
        let output_len = (*lock).len();

        println!("Process; In: {}, Out: {}", input_len, output_len);
      }

      let frame = {
        let mut lock = self.input.lock().unwrap(); // Fatal error.
        lock.pop_front()
      };

      let frame = match frame {
        Some(f) => f,
        None => {
          thread::sleep(Duration::from_millis(100));
          continue;
        },
      };

      println!("Process...");

      let points = parse_points(frame);

      let mut lock = self.output.lock().unwrap(); // Fatal error.

      for point in points {
        (*lock).push_back(point);
      }
    }

    /*loop {
      let mut frames = Vec::new();
      {
        //while let Some(frame) = lock.pop_front() {
        //  frames.push(frame);
        //  //if frames.len() > 100 { break; } // TODO: Prevent unbounded growth.
        //}

        loop {
          let mut lock = self.input.lock().unwrap(); // Fatal error.
          let frame = lock.pop_front();

          match frame {
            None => break, // Ran out of frames
            Some(frame) => frames.push(frame),
          }
        }

      }

      let mut points = Vec::new();

      for frame in frames {
        let frame_points = parse_points(frame);
        points.extend(frame_points);
      }

      {
        // TODO: Prevent over-fill.
        for point in points {
          let mut lock = self.output.lock().unwrap(); // Fatal error.
          (*lock).push_back(point);
        }
      }
    }*/
  }
}

/// Parse raw point bytes into structured Points.
fn parse_points(dac_frame: DacFrame) -> Vec<DrawPoint> {
  let mut reader = Cursor::new(dac_frame.point_data);
  let mut points : Vec<DrawPoint> = Vec::new();

  for _i in 0 .. dac_frame.num_points {
    let _control = reader.read_u16::<LittleEndian>().unwrap();
    let x = map_x(reader.read_i16::<LittleEndian>().unwrap(), 600);
    let y = map_y(reader.read_i16::<LittleEndian>().unwrap(), 600);
    let _i = reader.read_u16::<LittleEndian>().unwrap();
    let r = map_color(reader.read_u16::<LittleEndian>().unwrap());
    let g = map_color(reader.read_u16::<LittleEndian>().unwrap());
    let b = map_color(reader.read_u16::<LittleEndian>().unwrap());
    let _u1 = reader.read_u16::<LittleEndian>().unwrap();
    let _u2 = reader.read_u16::<LittleEndian>().unwrap();

    points.push(DrawPoint { x: x, y: y, r: r, g: g, b: b });
  }

  points
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
