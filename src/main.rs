// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// A OpenGL emulator/visualizer for the EtherDream laser projector DAC.
// See http://ether-dream.com/protocol.html

extern crate byteorder;
extern crate clap;
extern crate glium_graphics;
extern crate graphics;
extern crate ilda;
extern crate net2;
extern crate piston;

mod dac;
mod error;
mod pipeline;
mod protocol;
mod render;

use clap::App;
use clap::Arg;
use dac::Dac;
use net2::UdpBuilder;
use pipeline::Pipeline;
use protocol::Broadcast;
use protocol::DacStatus;
use render::gl_window;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::sync::Arc;
use std::thread::sleep;
use std::thread;
use std::time::Duration;

const TCP_PORT : u16 = 7765;
const UDP_PORT : u16 = 7654;

/// Program runtime options
#[derive(Clone,Debug)]
pub struct RuntimeOpts {
  /// Print debugging information to STDOUT,
  /// eg. each message in the client/server protocol.
  pub debug_protocol: bool,

  /// Don't spawn a GUI.
  pub headless: bool,
}

impl RuntimeOpts {
  fn read() -> RuntimeOpts {
    let matches = App::new("etherdream-emulator")
        .arg(Arg::with_name("debug")
             .long("debug")
             .short("d")
             .help("Turns debugging output on")
             .takes_value(false)
             .required(false))
        .arg(Arg::with_name("headless")
             .long("headless")
             .help("Turns off the GUI")
             .takes_value(false)
             .required(false))
        .get_matches();

    RuntimeOpts {
      debug_protocol: matches.is_present("debug"),
      headless: matches.is_present("headless"),
    }
  }
}

fn main() {
  let args = RuntimeOpts::read();

  let pipeline = Arc::new(Pipeline::new());
  let pipeline2 = pipeline.clone();

  let dac = Arc::new(Dac::new(&args, pipeline.clone()));
  let dac2 = dac.clone();

  thread::spawn(|| broadcast_thread());
  thread::spawn(move || gl_window(dac2, pipeline2));
  thread::spawn(move || pipeline.process());

  dac.listen_loop();
}

/// Send a UDP broadcast announcing the EtherDream to the network.
fn broadcast_thread() {
  let udp = UdpBuilder::new_v4().unwrap();
  udp.reuse_address(true).unwrap();

  let socket = udp.bind("0.0.0.0:7654").unwrap();
  socket.set_broadcast(true).unwrap();

  let multicast_ip = Ipv4Addr::new(255, 255, 255, 255);
  let multicast_socket = SocketAddr::V4(SocketAddrV4::new(multicast_ip, UDP_PORT));

  let broadcast = Broadcast {
      mac_address: vec![1, 2, 3, 4, 5, 255],
      hw_revision: 9000,
      sw_revision: 8000,
      buffer_capacity: 1000,
      max_point_rate: 1234567890,
      status: DacStatus {
        protocol: 1,
        light_engine_state: 2,
        playback_state: 3,
        source: 4,
        light_engine_flags: 5,
        playback_flags: 6,
        source_flags: 7,
        buffer_fullness: 8,
        point_rate: 9,
        point_count: 10,
      },
  };

  loop {
    sleep(Duration::from_secs(1));
    socket.send_to(&broadcast.serialize(), multicast_socket).unwrap();
  }
}

