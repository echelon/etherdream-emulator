// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// See http://ether-dream.com/protocol.html

use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;

const TCP_PORT : u16 = 7765;
const UDP_PORT : u16 = 7654;

// 20 bytes
pub struct DacStatus {
  pub protocol : u8,
  pub light_engine_state : u8,
  pub playback_state : u8,
  pub source : u8,
  pub light_engine_flags : u16,
  pub playback_flags : u16,
  pub source_flags : u16,
  pub buffer_fullness : u16,
  pub point_rate : u32,
  pub point_count : u32,
}

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

fn main() {
  let mut socket = UdpSocket::bind("0.0.0.0:7654").unwrap(); // TODO
  socket.set_broadcast(true).unwrap(); // TODO

  let multicast_ip = Ipv4Addr::new(255, 255, 255, 255); 
  let multicast_socket = SocketAddr::V4(SocketAddrV4::new(multicast_ip, UDP_PORT));

  let broadcast = Broadcast::new();

  loop {
    sleep(Duration::from_secs(1));
    println!("Sending multicast...");
    socket.send_to(&broadcast.serialize(), multicast_socket);
  }
}

