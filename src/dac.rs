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
    let mut buf = [0u8; 2048];

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
            println!("Bytes: {}, {}, {}", buf[0], buf[1], buf[2]);
            let num = read_u16(buf[1 .. 2]);
            let points = self.parse_points(&buf);
            Command::Data { num_points: num, points: points }
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

  fn parse_points(&self, buf: &[u8]) -> Vec<Point> {
    Vec::new() // TODO
  }

  fn write(&self, stream: &mut TcpStream, command: u8) {
    let write_result = stream.write(
      &DacResponse::new(ResponseState::Ack, command, self.state.clone()).serialize());
    match write_result {
      Ok(size) => { println!("Write: {}", size); },
      Err(_) => { println!("Write error."); },
    };
  }
}

// TODO/FIXME: Does Rust's casting use 2's complement? Do some maths.
fn read_i16(bytes: [u8; 2]) -> i16 {                                              
  (((bytes[0] as u16) << 8) | (bytes[1] as u16)) as i16 
}

fn read_u16(bytes: [u8; 2]) -> u16 {
  ((bytes[0] as u16) << 8) | (bytes[1] as u16)
}

