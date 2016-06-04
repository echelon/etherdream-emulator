// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// See http://ether-dream.com/protocol.html

extern crate net2;

mod protocol;

use net2::TcpBuilder;
use net2::UdpBuilder;
use protocol::DacResponse;
use protocol::ResponseState;
use protocol::DacStatus;
use std::io::Read;
use std::io::Write;
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

fn main() {
  thread::spawn(|| broadcast_thread());
  thread::spawn(|| dac_thread());

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

fn dac_thread() {
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
              Ok(size) => { println!("Read C: {}", size); },
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

