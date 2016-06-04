// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Some of the documentation text is taken directly from the Etherdream
// website, and the copyright belongs to Jacob Potter.
// See http://ether-dream.com/protocol.html

/** The DAC periodically sends state information. */
pub struct DacStatus {
  pub protocol: u8,
  pub light_engine_state: u8,
  pub playback_state: u8,
  pub source: u8,
  pub light_engine_flags: u16,
  pub playback_flags: u16,
  pub source_flags: u16,
  pub buffer_fullness: u16,
  pub point_rate: u32,
  pub point_count: u32,
}

impl DacStatus {
  pub fn empty() -> DacStatus {
    DacStatus {
      protocol: 0,
      light_engine_state: 0,
      playback_state: 0,
      source: 0,
      light_engine_flags: 0,
      playback_flags: 0,
      source_flags: 0,
      buffer_fullness: 0,
      point_rate: 0,
      point_count: 0,
    }
  }

  // FIXME: Serialization massively sucks.
  pub fn serialize(&self) -> Vec<u8> {
    let mut v = Vec::new();
    v.push(self.protocol);
    v.push(self.light_engine_state);
    v.push(self.playback_state);
    v.push(self.source);
    v.push(0); // TODO
    v.push(0);
    v.push(0);
    v.push(0);
    v.push(0);
    v.push(0);
    v.push(0);
    v.push(0);
    v.push(0);
    v.push(0);
    v.push(0);
    v.push(0);
    v.push(0);
    v.push(0);
    v.push(0);
    v.push(0);
    v
  }
}

struct BeginCommand {
  command: u8, // 'b' (0x62)
  low_water_mark: u16, // currently unused.
  point_rate: u32,
}

/*impl BeginCommand {
  pub fn parse(bytes: [u8]) -> BeginCommand {
    BeginCommand {
      command
    }
  }
}*/

struct DataCommand {
  command: u8, // 'd' (0x64)
  num_points: u16,
  dac_points: Vec<Point>,
}

struct Point {
  control: u16,
  x: u16,
  y: u16,
  i: u16,
  r: u16,
  g: u16,
  b: u16,
  u1: u16,
  u2: u16,
}

pub struct DacResponse {
  /**
   * Response can be any of the following:
   *
   * ACK - 'a' (0x61) - The previous command was accepted.
   * NAK - Full - 'F' (0x46) - The write command could not be performed
   *       because there was not enough buffer space when it was
   *       received.
   * NAK - Invalid - 'I' (0x49) - The command contained an invalid
   *       command byte or parameters.
   * NAK - Stop Condition - '!' (0x21) - An emergency-stop condition
   *       still exists.
   */
  response: u8,

  /**
   * In the case of ACK/NAK responses, "command" echoes back the command
   * to which the response is sent. (Commands are always sent in order,
   * so this field exists for sanity-checking on the host side.) 
   */
  command: u8,

  /** State of the DAC. */
  dac_status: DacStatus,
}

impl DacResponse {
  pub fn new(response: u8, command: u8, dac_status: DacStatus) -> DacResponse {
    DacResponse {
      response: response,
      command: command,
      dac_status: dac_status,
    }
  }

  pub fn info() -> DacResponse {
    DacResponse {
      response: 0x61,
      command: 0x3f, // '?'
      dac_status: DacStatus::empty(),
    }
  }

  pub fn serialize(&self) -> Vec<u8> {
    let mut vec = Vec::new();
    vec.push(self.response);
    vec.push(self.command);
    vec.extend(self.dac_status.serialize());
    vec
  }
}

