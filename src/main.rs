// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// A OpenGL emulator/visualizer for the EtherDream laser projector DAC.
// See http://ether-dream.com/protocol.html

extern crate byteorder;
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
use net2::UdpBuilder;
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

