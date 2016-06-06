// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use protocol::DacResponse;
use protocol::Command;
use protocol::DacStatus;
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
          self.read_command(&mut stream);
          self.write(&mut stream, 0x64);
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
        println!("Read: {}", size); 

        // TODO: Implement all commands
        match buf[0] {
          0x64 => {
            println!("Read data");
            Command::Data
          },
          0x70 => {
            println!("Read prepare");
            Command::Prepare
          },
          0x62 => {
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

  fn read_points(&self) {
    // TODO
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

