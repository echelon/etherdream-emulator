// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use dac::DacFrame;
use error::EmulatorError;
use protocol::Point;
use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

/// A separate thread to consume raw points off the wire and translate them into
/// graphical points ready to render. This takes load off the DAC thread as well
/// as the OpenGL/drawing thread.
pub struct Pipeline {
  input: Mutex<VecDeque<DacFrame>>,
  output: Mutex<VecDeque<Point>>,
  frame_limit: usize,
  point_limit: usize,
}

impl Pipeline {
  /// CTOR.
  pub fn new() -> Pipeline {
    Pipeline {
      input: Mutex::new(VecDeque::new()),
      output: Mutex::new(VecDeque::new()),
      frame_limit: 10,
      point_limit: 5_000,
    }
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
                 -> Result<Vec<Point>, EmulatorError> {
    let mut buf = Vec::new();
    let mut lock = self.output.lock()?;

    while buf.len() < num_points {
      match (*lock).pop_front() {
        None => return Ok(buf), // Return fewer frames than asked for.
        Some(frame) => buf.push(frame),
      }
    }
    Ok(buf)
  }

  /// Run by a separate thread from network and graphics.
  pub fn process(&self) -> ! {
    loop {
      let frame = {
        let mut lock = self.input.lock().unwrap(); // Fatal error.
        lock.pop_front()
      };

      let frame = match frame {
        Some(f) => f,
        None => {
          thread::sleep(Duration::from_millis(100)); // No work to do.
          continue;
        },
      };

      let points = parse_points(frame);

      let mut lock = self.output.lock().unwrap(); // Fatal error.

      for point in points {
        if (*lock).len() > self.point_limit {
          break; // TODO: Don't generate extra wasted points if over limit.
        }
        (*lock).push_back(point);
      }
    }
  }
}

/// Parse raw point bytes into structured Points.
#[inline]
fn parse_points(dac_frame: DacFrame) -> Vec<Point> {
  let mut reader = Cursor::new(dac_frame.point_data);
  let mut points : Vec<Point> = Vec::new();

  for _i in 0 .. dac_frame.num_points {
    // TODO: Consider moving point/color transforms from the gfx thread here.
    points.push(Point {
      control: reader.read_u16::<LittleEndian>().unwrap(),
      x:       reader.read_i16::<LittleEndian>().unwrap(),
      y:       reader.read_i16::<LittleEndian>().unwrap(),
      i:       reader.read_u16::<LittleEndian>().unwrap(),
      r:       reader.read_u16::<LittleEndian>().unwrap(),
      g:       reader.read_u16::<LittleEndian>().unwrap(),
      b:       reader.read_u16::<LittleEndian>().unwrap(),
      u1:      reader.read_u16::<LittleEndian>().unwrap(),
      u2:      reader.read_u16::<LittleEndian>().unwrap(),
    })
  }
  points
}
