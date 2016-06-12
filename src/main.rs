// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// A OpenGL emulator/visualizer for the EtherDream laser projector DAC.
// See http://ether-dream.com/protocol.html

extern crate byteorder;
extern crate glium;
extern crate glium_graphics;
extern crate graphics;                                                          
extern crate ilda;
extern crate net2;
extern crate piston;
extern crate rand;

mod dac;
mod protocol;
mod render;

use dac::Dac;
use net2::TcpBuilder;
use net2::UdpBuilder;
use protocol::DacResponse;
use protocol::DacStatus;
use protocol::Point;
use protocol::ResponseState;
use rand::Rng;
use render::gl_window;
use std::io::Read;
use std::io::Write;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::TcpListener;
use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::thread::sleep;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const TCP_PORT : u16 = 7765;
const UDP_PORT : u16 = 7654;

// 16 bytes + dac status -> 36 bytes
pub struct Broadcast {
  pub mac_address : Vec<u8>, // TODO: fixed size 
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
      status: DacStatus::empty(),
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

fn main() {
  let dac = Arc::new(Dac::new());
  let dac2 = dac.clone();

  thread::spawn(|| broadcast_thread());
  thread::spawn(move || gl_window(dac2));

  dac.listen_loop();
}

/// Send a UDP broadcast announcing the EtherDream to the network.
fn broadcast_thread() {
  let udp = UdpBuilder::new_v4().unwrap();
  udp.reuse_address(true).unwrap();

  let mut socket = udp.bind("0.0.0.0:7654").unwrap();
  socket.set_broadcast(true).unwrap();

  let multicast_ip = Ipv4Addr::new(255, 255, 255, 255); 
  let multicast_socket = SocketAddr::V4(SocketAddrV4::new(multicast_ip, UDP_PORT));

  let broadcast = Broadcast::new();

  loop {
    sleep(Duration::from_secs(1));
    socket.send_to(&broadcast.serialize(), multicast_socket);
  }
}

