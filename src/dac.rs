// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use RuntimeOpts;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use error::ClientError;
use pipeline::Pipeline;
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
use std::time::Instant;

/// Size of a single point in bytes.
const POINT_SIZE : usize = 18;

type PointQueue = VecDeque<Point>;

/// Points sent from the Dac in a single DATA command payload.
pub struct DacFrame {
  pub num_points: u16,
  pub point_data: Vec<u8>,
}

pub struct Dac {
  /// Runtime arguments supplied to the program.
  opts: RuntimeOpts,

  // TODO: Refactor so locking not required. Only a single thread
  // needs this.
  /// Runtime state of the virtual dac.
  status: RwLock<DacStatus>,

  /// Queue of points read from a client.
  points: Mutex<PointQueue>,

  /// Maximum size of the queue. (TODO: Notes on [non-]blocking.)
  queue_limit: usize,

  /// Point pipeline
  pipeline: Arc<Pipeline>,
}

impl Dac {
  pub fn new(opts: &RuntimeOpts, pipeline: Arc<Pipeline>) -> Dac {
    Dac {
      opts: opts.clone(),
      status: RwLock::new(DacStatus::empty()),
      points: Mutex::new(PointQueue::new()),
      queue_limit: 60_000,
      pipeline: pipeline,
    }
  }

  pub fn listen_loop(&self) {
    loop { self.listen(); }
  }

  pub fn listen(&self) {
    // TODO: Set timeout on listener
    let listener = TcpListener::bind("0.0.0.0:7765").unwrap();

    match listener.accept() {
      Err(e) => {
        println!("Error: {:?}", e);
      },
      Ok((mut stream, _socket_addr)) => {
        self.log("Connected!");

        // Write info
        self.write(&mut stream, &Command::Ping);

        loop {
          // Read-write loop
          let start = Instant::now();
          let command = self.read_command(&mut stream);

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
              return;
            },
          }

          // TODO:
          //  - Dac thread consumes protocol
          //  - Point thread converts into points
          //  - Draw thread consumes points


          /*match command {
            Command::Data { points, .. } => {
              let _r = self.pipeline.enqueue(points); // TODO: Error handling!
              self.enqueue_points(points);
            },
            _ => {},
          }*/
          /*

          Original Times:

            Elapsed: Duration { secs: 0, nanos: 12799693 }
            Elapsed: Duration { secs: 0, nanos: 5126380 }
            Elapsed: Duration { secs: 0, nanos: 5191206 }
            Elapsed: Duration { secs: 0, nanos: 5271904 }
            Elapsed: Duration { secs: 0, nanos: 5350840 }
            Elapsed: Duration { secs: 0, nanos: 5385611 }
            Elapsed: Duration { secs: 0, nanos: 5414422 }
            Elapsed: Duration { secs: 0, nanos: 5416543 }

          Without parsing:

            Elapsed: Duration { secs: 0, nanos: 12575459 }
            Elapsed: Duration { secs: 0, nanos: 12713894 }
            Elapsed: Duration { secs: 0, nanos: 12857095 }
            Elapsed: Duration { secs: 0, nanos: 13090941 }
            Elapsed: Duration { secs: 0, nanos: 13602049 }
            Elapsed: Duration { secs: 0, nanos: 14051671 }

          Without parsing x2:

            Elapsed: Duration { secs: 0, nanos: 5055916 }
            Elapsed: Duration { secs: 0, nanos: 5100141 }
            Elapsed: Duration { secs: 0, nanos: 5122131 }
            Elapsed: Duration { secs: 0, nanos: 6235534 }
            Elapsed: Duration { secs: 0, nanos: 6257061 }
            Elapsed: Duration { secs: 0, nanos: 6432697 }
            Elapsed: Duration { secs: 0, nanos: 6534558 }
            Elapsed: Duration { secs: 0, nanos: 6722200 }
            Elapsed: Duration { secs: 0, nanos: 7244421 }
            Elapsed: Duration { secs: 0, nanos: 7733401 }
          */

          let elapsed = start.elapsed();

          println!("Elapsed: {:?}", elapsed);
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
    match self.points.lock() {
      Err(_) => {
        false
      },
      Ok(mut queue) => {
        if queue.len() + points.len() > self.queue_limit {
          println!("Queue max reached.");
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
    let mut buf = [0u8; 2048]; // TODO: Better buffer size.

    match stream.read(&mut buf) {
      Err(_) => {
        self.log("Read error.");
        Command::Unknown{ command: 0u8 } // TODO: Return error instead
      },
      Ok(size) => {
        // TODO: Implement all commands
        match buf[0] {
          COMMAND_DATA => {
            let (num_points, point_bytes) =
                self.read_point_data(stream, buf, size);

            //let points = self.parse_points(num_points, point_bytes);
            // FIXME: Error handling!
            // TODO: Refactor.
            let frame = DacFrame {
              num_points: num_points,
              point_data: point_bytes,
            };

            let _r = self.pipeline.enqueue(frame);

            Command::Data { num_points: num_points, points: Vec::new() }
          },
          COMMAND_PREPARE => {
            self.log("Read prepare");
            Command::Prepare
          },
          COMMAND_BEGIN => {
            match parse_begin(&buf) {
              Ok(b) => b,
              Err(e) => Command::Unknown { command: buf[0] },
            }
          },
          _ => {
            self.log("Read unknown");
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
                     read_size: usize) -> (u16, Vec<u8>) {
    self.log("Reading data");

    let num_points = read_u16(&buf[1 .. 3]);

    self.log(&format!("Reading {} points.", num_points));

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

    for _i in 0 .. num_points {
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

  /// Write ACK back to client.
  fn write(&self, stream: &mut TcpStream, command: &Command) {
    let status = match self.status.read() {
      Err(_) => { DacStatus::empty() },
      Ok(s) => { s.clone() },
    };

    let write_result = stream.write(
      &DacResponse::new(ResponseState::Ack, command.value(), status).serialize());

    match write_result {
      Err(_) => {
        println!("Write error.");
      },
      Ok(size) => {
        self.log(&format!("Wrote {} ACK, {} bytes", command.name(), size));
      },
    };
  }

  fn log(&self, message: &str) {
    if self.opts.debug_protocol {
      println!("{}", message);
    }
  }
}

// TODO: Use the byteorder library instead.
fn read_u16(bytes: &[u8]) -> u16 {
  ((bytes[0] as u16) << 8) | (bytes[1] as u16)
}

/// Parse a 'begin' command.
pub fn parse_begin(bytes: &[u8]) -> Result<Command, ClientError> {
  let mut reader = Cursor::new(bytes);
  let b = try!(reader.read_u8()); // FIXME

  if b != COMMAND_BEGIN {
    return Err(ClientError::ParseError);
  }

  let lwm = try!(reader.read_u16::<LittleEndian>()); // FIXME
  let pr = try!(reader.read_u32::<LittleEndian>()); // FIXME

  Ok(Command::Begin {
    low_water_mark: lwm,
    point_rate: pr,
  })
}

