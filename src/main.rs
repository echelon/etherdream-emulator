// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// See http://ether-dream.com/protocol.html

//extern crate glutin;
extern crate rand;
extern crate net2;

extern crate graphics;                                                          
extern crate glium;
extern crate glium_graphics;
extern crate piston;

use glium::DisplayBuild;
use glium_graphics::{                                                           
    Flip, Glium2d, GliumWindow, OpenGL, Texture, TextureSettings
};

use piston::input::*; 
use piston::window::WindowSettings; 
use graphics::draw_state::Blend; 

mod protocol;

use net2::TcpBuilder;
use rand::Rng;
use std::time::Instant;
use net2::UdpBuilder;
use protocol::DacResponse;
use protocol::ResponseState;
use protocol::DacStatus;
use protocol::Point;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::TcpListener;
use std::net::UdpSocket;
use std::thread::sleep;
use std::thread;
use std::time::Duration;

const TCP_PORT : u16 = 7765;
const UDP_PORT : u16 = 7654;

// 16 bytes + dac status -> 36 bytes
pub struct Broadcast {
  pub mac_address : Vec<u8>, // TODO: better type
  //uint8_t mac_address[6];
  pub hw_revision : u16,
  pub sw_revision : u16,
  pub buffer_capacity : u16,
  pub max_point_rate : u32,
  pub status : DacStatus,
}

impl Broadcast {
  pub fn new() -> Broadcast {
    Broadcast {
      mac_address: Vec::new(),
      hw_revision: 0u16,
      sw_revision: 0u16,
      buffer_capacity: 0u16,
      max_point_rate: 0u32,
      status: DacStatus {
        protocol: 0u8,
        light_engine_state: 0u8,
        playback_state: 0u8,
        source: 0u8,
        light_engine_flags: 0u16,
        playback_flags: 0u16,
        source_flags: 0u16,
        buffer_fullness: 0u16,
        point_rate: 0u32,
        point_count: 0u32,
      }
    }
  }

  pub fn serialize(&self) -> Vec<u8> {
    let mut vec = Vec::new();
    for i in 0..36 {
      vec.push(0);
    }
    vec
  }
}

pub struct TimedPoint {
  pub point: Point,
  pub instant: Instant,
}

impl TimedPoint {
  pub fn new(point: Point) -> TimedPoint {
    TimedPoint {
      point: point,
      instant: Instant::now(),
    }
  }

  pub fn can_draw(&self) -> bool {
    self.instant.elapsed() < Duration::from_millis(100)
  }
}

pub struct PointBuffer {
  buffer: Vec<TimedPoint>,
  next: usize,
  capacity: usize,
}

#[derive(Clone)]
pub struct AtomicPointBuffer {
  holder: Arc<RwLock<PointBuffer>>,
}

impl PointBuffer {
  pub fn new() -> PointBuffer {
    PointBuffer {
      buffer: Vec::with_capacity(2),
      next: 0,
      capacity: 2,
    }
  }

  pub fn add(&mut self, point: TimedPoint) {
    self.buffer.insert(self.next, point);
    self.next = (self.next + 1) % self.capacity; // FIXME: Capacity
  }
  pub fn read(&self) -> &Vec<TimedPoint> {
    &self.buffer
  }
}

/*impl AtomicPointBuffer {
  pub fn new() -> AtomicPointBuffer {
    let buffer = PointBuffer::new();
    AtomicPointBuffer {
      holder: Arc::new(RwLock::new(buffer))
    }
  }

  pub get(&self) -> &mut PointBuffer {
  }
}*/

fn main() {
  let mut buffer = Arc::new(RwLock::new(PointBuffer::new()));

  let buffer2 = buffer.clone();
  let buffer3 = buffer.clone();

  thread::spawn(|| broadcast_thread());
  thread::spawn(|| dac_thread(buffer2));
  thread::spawn(|| gl_window(buffer3));

  loop {
    sleep(Duration::from_secs(10)); // TODO: Join other threads
  }
}

fn broadcast_thread() {
  let udp = UdpBuilder::new_v4().unwrap(); // TODO
  udp.reuse_address(true).unwrap(); // TODO

  let mut socket = udp.bind("0.0.0.0:7654").unwrap(); // TODO

  socket.set_broadcast(true).unwrap(); // TODO

  let multicast_ip = Ipv4Addr::new(255, 255, 255, 255); 
  let multicast_socket = SocketAddr::V4(SocketAddrV4::new(multicast_ip, UDP_PORT));

  let broadcast = Broadcast::new();

  loop {
    sleep(Duration::from_secs(1));
    //println!("Sending multicast...");
    socket.send_to(&broadcast.serialize(), multicast_socket);
  }
}

fn dac_thread(buffer: Arc<RwLock<PointBuffer>>) {
  //tcp.reuse_address(true).unwrap(); // TODO

  //socket.set_broadcast(true).unwrap(); // TODO
  //let multicast_ip = Ipv4Addr::new(255, 255, 255, 255); 
  //let multicast_socket = SocketAddr::V4(SocketAddrV4::new(multicast_ip, UDP_PORT));
  //let mut stream = socket.to_tcp_stream().unwrap(); // TODO
  //let broadcast = Broadcast::new();
  //

  /*let tcp = TcpBuilder::new_v4().unwrap(); // TODO
  let mut socket = tcp.bind("0.0.0.0:7765").unwrap(); // TODO
  let mut listener = socket.to_tcp_listener().unwrap(); // TODO */

  loop {
    sleep(Duration::from_millis(50)); 

    match (*buffer).write() {
      Err(_) => {},
      Ok(mut pb) => {
        println!("Adding point!");
        pb.add(TimedPoint::new(Point::random()));
      }
    };

  }





  let listener = TcpListener::bind("0.0.0.0:7765").unwrap();
  listener.set_ttl(500); // FIXME: Assume millisec

  loop {
    sleep(Duration::from_secs(1));
    println!("Dac thread.");
    match listener.accept() {
      Err(e) => {
        println!("Error: {:?}", e);
      },
      Ok((mut stream, socket_addr)) => {
        println!("Connected!");
        //stream.set_ttl(500).unwrap(); // FIXME: Assume millisec



        // FIXME: THIS IS ABSOLUTE GARBAGE. MAKE A STATE MACHINE.
        loop {
          let mut state = DacStatus::empty();
          //let mut bytes = [0u8; 56]; // TODO: Better buffer
          let mut bytes = [0u8; 2048]; // TODO: Better buffer

          // ***** A *****
          let mut write_result = stream.write(&DacResponse::info().serialize());
          match write_result {
            Ok(size) => { println!("Write A: {}", size); },
            Err(_) => { println!("Write error A."); },
          };

          // ***** B *****
          // TODO: DON'T IGNORE PREPARE COMMAND (p / 0x70)
          let mut read_result = stream.read(&mut bytes);
          match read_result {
            Ok(size) => { println!("Read B: {}", size); },
            Err(_) => { println!("Read error B."); },
          };
          write_result = stream.write(
              &DacResponse::new(ResponseState::Ack, 0x70, state.clone()).serialize());
          match write_result {
            Ok(size) => { println!("Write B: {}", size); },
            Err(_) => { println!("Write error B."); },
          };


          // ***** C ***** "Data"
            read_result = stream.read(&mut bytes);
            match read_result {
              Ok(size) => { println!("Read C: {}", size); },
              Err(_) => { println!("Read error C."); },
            };
            write_result = stream.write(
                &DacResponse::new(ResponseState::Ack, 0x64, state.clone()).serialize());
            match write_result {
              Ok(size) => { println!("Write C: {}", size); },
              Err(_) => { println!("Write error C."); },
            };

          // ***** D *****: "Begin" 
            read_result = stream.read(&mut bytes);
            match read_result {
              Ok(size) => { println!("Read D: {}", size); },
              Err(_) => { println!("Read error D."); },
            };
            write_result = stream.write(
                &DacResponse::new(ResponseState::Ack, 0x62, state.clone()).serialize());
            match write_result {
              Ok(size) => { println!("Write D: {}", size); },
              Err(_) => { println!("Write error D."); },
            };


          // ***** C ***** "More data"
          loop {
            read_result = stream.read(&mut bytes);
            match read_result {
              Ok(size) => { 
                println!("Read C: {}", size); 

                match (*buffer).write() {
                  Err(_) => {},
                  Ok(mut pb) => {
                    println!("Adding point!");
                    pb.add(TimedPoint::new(Point::random()));
                  }
                }

              },
              Err(_) => { println!("Read error C."); },
            };
            write_result = stream.write(
                &DacResponse::new(ResponseState::Ack, 0x64, state.clone()).serialize());
            match write_result {
              Ok(size) => { println!("Write C: {}", size); },
              Err(_) => { println!("Write error C."); },
            };
          }
        }

      },
    }
    //println!("Sending multicast...");
    //socket.send_to(&broadcast.serialize(), multicast_socket);
  }
}

fn gl_window(buffer: Arc<RwLock<PointBuffer>>) {
  let opengl = OpenGL::V3_2;
  let (w, h) = (1280, 960);
  let ref mut window: GliumWindow =
    WindowSettings::new("glium_graphics: image_test", [w, h])
    .exit_on_esc(true).opengl(opengl).build().unwrap();


  let mut g2d = Glium2d::new(opengl, window); 
  while let Some(e) = window.next() { 
    if let Some(args) = e.render_args() { 
      use graphics::*;

      let mut target = window.draw();
      g2d.draw(&mut target, args.viewport(), |c, g| {

        let point_transform = c.transform.scale(0.05, 0.05);
        let mut rng = rand::thread_rng();

        clear([1.0; 4], g);

        // Background
        Rectangle::new([0.0, 0.0, 0.0, 1.0])
          .draw([0.0, 0.0, 1280.0, 1280.0], &c.draw_state, c.transform, g);

        match (*buffer).read() {
          Err(_) => {},
          Ok(pb) => {
            let points = pb.read();

            for timed_point in points {
              /*if !timed_point.can_draw() {
                continue;
              }*/

              let x = map_x(timed_point.point.x, 1280);
              let y = map_y(timed_point.point.y, 960);

              println!("{}, {}", x, y);
              println!("{}, {}", timed_point.point.x, timed_point.point.y);

              let r = rng.gen_range(0.0, 1.0);
              let gr = rng.gen_range(0.0, 1.0);
              let b = rng.gen_range(0.0, 1.0);

              Ellipse::new([r, gr, b, 1.0])
                .draw([
                      x, 
                      y,
                      10.0, 
                      10.0,
                ], 
                &c.draw_state, c.transform, g);

            }
          },
        }

        /*let x = 100.0;
        let y = 100.0;


        Ellipse::new([1.0, 1.0, 1.0, 1.0])
          .draw([x, y, x, y, ], 
          &c.draw_state, transform, g);

        //clear([0.0, 0.0, 1.0, 1.0], g);
        //g.clear_color([0.0, 1.0, 0.0, 1.0]);*/
        sleep(Duration::from_millis(500)); 

      });

      target.finish().unwrap();
    }
  }
}

pub fn map_x(x: u16, width: u16) -> f64 {
  let scale = width as f64 / 65535.0;
  x as f64 * scale
}

pub fn map_y(y: u16, height: u16) -> f64 {
  let scale = height as f64 / 65535.0;
  y as f64 * scale
}
