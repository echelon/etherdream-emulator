// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use protocol::DacResponse;
use protocol::Command;
use protocol::DacStatus;
use protocol::COMMAND_PREPARE;
use protocol::COMMAND_BEGIN;
use protocol::COMMAND_DATA;
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::Read;
use std::io::Write;
use protocol::Point;
use protocol::ResponseState;

pub struct Dac {
  state: DacStatus,
}

/// Size of a single point in bytes.
const POINT_SIZE : usize = 18;

impl Dac {
  pub fn new() -> Dac {
    Dac {
      state: DacStatus::empty(),
    }
  }

  pub fn listen(&self) {
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
            Command::Data { .. } => {
              self.write(&mut stream, COMMAND_DATA);
            },
            _ => {
            },
          }
        }
      },
    };
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
        println!("Read bytes: {}", size);

        // TODO: Implement all commands
        match buf[0] {
          COMMAND_DATA => {
            println!("Read data");
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

    //println!("  - Num points: {}", num_points);
    //println!("  - Already read bytes: {}", read_size);
    //println!("  - Total size: {}", total_size);

    while total_size > already_read {
      let mut read_buf = [0u8; 2048];

      match stream.read(&mut read_buf) {

        Err(_) => {
          println!("READ ERROR."); // TODO Result<T,E>
          return (0, Vec::new());
        },
        Ok(size) => {
          //println!("    - Read: {}", size);
          point_buf.extend_from_slice(&read_buf[0 .. size]);
          already_read += size;
          //println!("    - Already read bytes: {}", already_read);
        },
      }
    }

    //println!("  - Read done!");
    (num_points, point_buf)
  }

  /// Parse raw point bytes into structured Points.
  fn parse_points(&self, num_points: u16, point_data: Vec<u8>) 
      -> Vec<Point> {
    let mut points : Vec<Point> = Vec::new();

    for i in 0 .. num_points {
      let j = i as usize * POINT_SIZE;
      points.push(Point {
        control: read_u16(&point_data[j .. j+2]),
        x:       read_u16(&point_data[j+2 .. j+4]),
        y:       read_u16(&point_data[j+4 .. j+6]),
        i:       read_u16(&point_data[j+6 .. j+8]),
        r:       read_u16(&point_data[j+8 .. j+10]),
        g:       read_u16(&point_data[j+10 .. j+12]),
        b:       read_u16(&point_data[j+12 .. j+14]),
        u1:      read_u16(&point_data[j+14 .. j+16]),
        u2:      read_u16(&point_data[j+16 .. j+18]),
      })
    }

    points
  }

  fn write(&self, stream: &mut TcpStream, command: u8) {
    let write_result = stream.write(
      &DacResponse::new(ResponseState::Ack, command,
                        self.state.clone()).serialize());
    match write_result {
      Ok(size) => { println!("Write: {}", size); },
      Err(_) => { println!("Write error."); },
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

