
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

struct DacResponse {
  response: u8,
  command: u8,
  dac_status: DacStatus,
}

