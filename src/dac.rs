// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use RuntimeOpts;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use error::EmulatorError;
use pipeline::Pipeline;
use protocol::COMMAND_BEGIN;
use protocol::COMMAND_DATA;
use protocol::COMMAND_PREPARE;
use protocol::Command;
use protocol::DacResponse;
use protocol::DacStatus;
use protocol::ResponseState;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

/// Size of a single point in bytes.
const POINT_SIZE : usize = 18;

/// Points sent from the Dac in a single DATA command payload.
pub struct DacFrame {
  pub num_points: u16,
  pub point_data: Vec<u8>,
}

pub struct Dac {
  /// Runtime arguments supplied to the program.
  opts: RuntimeOpts,

  // TODO: Refactor so locking not required. Only a single thread needs this.
  /// Runtime state of the virtual dac.
  status: RwLock<DacStatus>,

  /// Point pipeline (point queue)
  pipeline: Arc<Pipeline>,
}

impl Dac {
  pub fn new(opts: &RuntimeOpts, pipeline: Arc<Pipeline>) -> Dac {
    Dac {
      opts: opts.clone(),
      status: RwLock::new(DacStatus::empty()),
      pipeline: pipeline,
    }
  }

  /// Run the dac server. Accepts a connection, then begins the dac state
  /// machine to handle points sent by the client.
  pub fn run(&self) {
    loop {
      self.reset_status();
      let _r = self.listen(); // TODO: handle errors.
    }
  }

  pub fn listen(&self) -> Result<(), EmulatorError> {
    let listener = TcpListener::bind("0.0.0.0:7765")?;

    let (mut stream, _socket_addr) = listener.accept()?;
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
    stream.set_write_timeout(Some(Duration::from_millis(100)))?;

    self.log("Connected!");

    // Write info
    self.write(&mut stream, &Command::Ping)?;

    // TODO: Refactor into proper state machine.
    loop {
      // Read-write loop
      let command = self.read_command(&mut stream)?;

      self.log(&format!("Read command: {}", command));

      match command {
        Command::Begin { .. } => {
          self.write(&mut stream, &command);
        },
        Command::Prepare => {
          self.write(&mut stream, &command);
        },
        Command::Data { .. } => {
          self.write(&mut stream, &command);
        },
        _ => {
          println!("Cannot send ack for unknown/unhandled command.");
          return Err(EmulatorError::UnknownCommand);
        },
      }
    }

    unreachable!()
  }

  fn read_command(&self, stream: &mut TcpStream)
      -> Result<Command, EmulatorError> {
    let mut buf = [0u8; 2048]; // TODO: Better buffer size.

    let size = stream.read(&mut buf)?;

    match buf[0] {
      COMMAND_DATA => {
        let (num_points, point_data) = self.read_point_data(stream, buf, size)?;
        let frame = DacFrame {
          num_points: num_points,
          point_data: point_data,
        };

        // TODO: Handle full buffer.
        let _r = self.pipeline.enqueue(frame);

        // TODO: Report buffer size to apply back pressure.
        //match self.status.try_write() {
        //  Err(_) => {},
        //  Ok(mut status) => {
        //    status.buffer_fullness = self.pipeline.queue_size();
        //  },
        //}

        Ok(Command::Data { num_points: num_points })
      },
      COMMAND_PREPARE => {
        Ok(Command::Prepare)
      },
      COMMAND_BEGIN => {
        // TODO: Include command code in error.
        parse_begin(&buf).map_err(|_| EmulatorError::UnknownCommand)
      },
      _ => {
        // TODO: Implement all commands
        self.log("Read unknown");
        Err(EmulatorError::UnknownCommand)
      },
    }
  }

  // TODO: Simplify and clean up
  /// Continue streaming point data payload.
  /// Returns the number of points as well as the point bytes.
  fn read_point_data(&self, stream: &mut TcpStream,
                     buf: [u8; 2048],
                     read_size: usize)
                     -> Result<(u16, Vec<u8>), EmulatorError> {
    let num_points = read_u16(&buf[1 .. 3]);

    self.log(&format!("Reading {} points.", num_points));

    let points_size = POINT_SIZE * num_points as usize;
    let total_size = points_size + 3usize; // 3 command header bytes

    let mut already_read = read_size;
    let mut point_buf : Vec<u8> = Vec::new();

    point_buf.extend_from_slice(&buf[3.. read_size]); // Omit 3 header bytes

    while total_size > already_read {
      let mut read_buf = [0u8; 2048];

      let size = stream.read(&mut read_buf)?;

      if size == 0 {
        // NB: If the client disconnects now, we can get stuck in a loop reading
        // zero bytes. Not sure why the socket doesn't report this error.
        return Err(EmulatorError::ClientError);
      }

      point_buf.extend_from_slice(&read_buf[0 .. size]);
      already_read += size;
    }

    Ok((num_points, point_buf))
  }

  /// Write ACK back to client.
  fn write(&self, stream: &mut TcpStream, command: &Command)
      -> Result<(), EmulatorError> {
    let status = self.status.read()?.clone();

    let response = &DacResponse::new(
      ResponseState::Ack, command.value(), status).serialize();

    let size = stream.write(response)?;

    Ok(())
  }

  /// Reset internal status.
  fn reset_status(&self) {
    let _r = self.status.try_write()
        .map(|mut status| *status = DacStatus::empty()); // Ignore lock errors.
  }

  fn log(&self, message: &str) {
    if self.opts.debug_protocol {
      // TODO: use logging crate or make a compile flag instead.
      println!("{}", message);
    }
  }
}

// TODO: Use the byteorder library instead.
#[inline]
fn read_u16(bytes: &[u8]) -> u16 {
  ((bytes[0] as u16) << 8) | (bytes[1] as u16)
}

/// Parse a 'begin' command.
#[inline]
pub fn parse_begin(bytes: &[u8]) -> Result<Command, EmulatorError> {
  let mut reader = Cursor::new(bytes);
  let b = reader.read_u8()?;

  if b != COMMAND_BEGIN {
    return Err(EmulatorError::ParseError);
  }

  Ok(Command::Begin {
    low_water_mark: reader.read_u16::<LittleEndian>()?,
    point_rate: reader.read_u32::<LittleEndian>()?,
  })
}

