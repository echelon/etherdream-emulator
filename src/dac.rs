// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use byteorder::{LittleEndian, ReadBytesExt};
use protocol::COMMAND_BEGIN;
use protocol::COMMAND_DATA;
use protocol::COMMAND_PREPARE;
use protocol::Command;
use protocol::DacResponse;
use protocol::DacStatus;
use protocol::Point;
use protocol::ResponseState;
use std::collections::VecDeque;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

/// Size of a single point in bytes.
const POINT_SIZE : usize = 18;

type PointQueue = VecDeque<Point>;

pub struct Dac {
  // TODO: Refactor so locking not required. Only a single thread
  // needs this.
  /// Runtime state of the virtual dac.
  status: RwLock<DacStatus>,

  /// Queue of points read from a client.
  points: Mutex<PointQueue>,

  /// Maximum size of the queue. (TODO: Notes on [non-]blocking.)
  queue_limit: usize,
}

impl Dac {
  pub fn new() -> Dac {
    Dac {
      status: RwLock::new(DacStatus::empty()),
      points: Mutex::new(PointQueue::new()),
      queue_limit: 60_000,
    }
  }

  pub fn listen_loop(&self) {
    loop { self.listen(); }
  }

  pub fn listen(&self) {
    // NB: Mutable only to change `status`
    let listener = TcpListener::bind("0.0.0.0:7765").unwrap();
    listener.set_ttl(500); // FIXME: I'm assuming millisec here.

    match listener.accept() {
      Err(e) => {
        println!("Error: {:?}", e);
      },
      Ok((mut stream, _socket_addr)) => {
        println!("Connected!");

        // Write info
        self.write(&mut stream, 0x3f);

        loop {
          // Read-write loop
          let command = self.read_command(&mut stream);

          match command {
            Command::Begin { .. } => {
              self.write(&mut stream, COMMAND_BEGIN);
            },
            Command::Prepare => {
              self.write(&mut stream, COMMAND_PREPARE);
            },
            Command::Data { num_points, points } => {
              self.enqueue_points(points);
              self.write(&mut stream, COMMAND_DATA);
            },
            _ => {
              println!("Unhandled command.");
              return;
            },
          }
        }
      },
    };
  }

  /// Drain points off the internal queue.
  pub fn drain_points(&self) -> Vec<Point> {
    match self.points.lock() {
      Err(_) => {
        println!("Error obtaining lock.");
        Vec::new()
      },
      Ok(mut queue) => {
        let mut points = Vec::new();
        while let Some(point) = queue.pop_front() {
          points.push(point);
        }
        points
      },
    }
  }

  /// Enqueue points. If the queue is full, reject and return false.
  fn enqueue_points(&self, points: Vec<Point>) -> bool {
    // NB: Mutable only to change `status`
    match self.points.lock() {
      Err(_) => {
        false
      },
      Ok(mut queue) => { 
        if queue.len() + points.len() > self.queue_limit {
          //println!("Queue max reached.");
          false
        } else {
          queue.extend(points); 
          match self.status.try_write() {
            Err(_) => {},
            Ok(mut status) => {
              status.buffer_fullness = queue.len() as u16;
            },
          }
          true
        }
      }
    }
  }

  fn read_command(&self, stream: &mut TcpStream) -> Command {
    let mut command_buf : Vec<u8> = Vec::new();
    let mut buf = [0u8; 2048]; // TODO: Better buffer size.

    match stream.read(&mut buf) {
      Err(_) => {
        println!("Read error.");
        Command::Unknown{ command: 0u8 } // TODO: Return error instead
      },
      Ok(size) => {
        //println!("Read bytes: {}", size);

        // TODO: Implement all commands
        match buf[0] {
          COMMAND_DATA => {
            //println!("Read data");
            let (num_points, point_bytes) = 
                self.read_point_data(stream, buf, size);

            let points = self.parse_points(num_points, point_bytes);

            Command::Data { num_points: num_points, points: points }
          },
          COMMAND_PREPARE => {
            println!("Read prepare");
            Command::Prepare
          },
          COMMAND_BEGIN => {
            println!("Read begin");
            Command::Begin { low_water_mark: 0, point_rate: 0 }
          },
          _ => {
            println!("Read unknown");
            Command::Unknown{ command: buf[0] }
          },
        }
      },
    }
  }

  // TODO: Simplify and clean up
  /// Continue streaming point data payload.
  /// Returns the number of points as well as the point bytes.
  fn read_point_data(&self, stream: &mut TcpStream, buf: [u8; 2048],
                     read_size: usize) 
      -> (u16, Vec<u8>) {

    let num_points = read_u16(&buf[1 .. 3]);
    let points_size = POINT_SIZE * num_points as usize;
    let total_size = points_size + 3usize; // 3 command header bytes

    let mut already_read = read_size;
    let mut point_buf : Vec<u8> = Vec::new();

    point_buf.extend_from_slice(&buf[3.. read_size]); // Omit 3 header bytes

    while total_size > already_read {
      let mut read_buf = [0u8; 2048];

      match stream.read(&mut read_buf) {

        Err(_) => {
          println!("READ ERROR."); // TODO Result<T,E>
          return (0, Vec::new());
        },
        Ok(size) => {
          point_buf.extend_from_slice(&read_buf[0 .. size]);
          already_read += size;
        },
      }
    }

    (num_points, point_buf)
  }

  /// Parse raw point bytes into structured Points.
  fn parse_points(&self, num_points: u16, point_data: Vec<u8>) 
      -> Vec<Point> {
    let mut reader = Cursor::new(point_data);
    let mut points : Vec<Point> = Vec::new();

    for i in 0 .. num_points {
      let j = i as usize * POINT_SIZE;

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

  fn write(&self, stream: &mut TcpStream, command: u8) {
    let status = match self.status.read() {
      Err(_) => { DacStatus::empty() },
      Ok(s) => { s.clone() },
    };

    let write_result = stream.write(
      &DacResponse::new(ResponseState::Ack, command, status).serialize());

    match write_result {
      Err(_) => { println!("Write error."); },
      Ok(size) => {},
    };
  }
}

// TODO/FIXME: Does Rust's casting use 2's complement? Do some maths.
fn read_i16(bytes: &[u8]) -> i16 {
  (((bytes[0] as u16) << 8) | (bytes[1] as u16)) as i16
}

fn read_u16(bytes: &[u8]) -> u16 {
  ((bytes[0] as u16) << 8) | (bytes[1] as u16)
}

