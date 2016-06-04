// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Some of the documentation text is taken directly from the Etherdream
// website, and the copyright belongs to Jacob Potter.
// See http://ether-dream.com/protocol.html

/** The DAC periodically sends state information. */

#[derive(Clone)]
pub struct DacStatus {
  pub protocol: u8,

  /**
   * The light engine is one of three state machines in the DAC.
   *
   * The states are:
   *
   *  - 0: Ready.
   *  - 1: Warmup. In the case where the DAC is also used for thermal
   *       control of laser apparatus, this is the state that is 
   *       entered after power-up.
   *  - 2: Cooldown. Lasers are off but thermal control is still active
   *  - 3: Emergency stop. An emergency stop has been triggered, either
   *       by an E-stop input on the DAC, an E-stop command over the
   *       network, or a fault such as over-temperature.
   *
   *  (Since thermal control is not implemented yet, it is not defined
   *  how transitions to and from the "Warmup" and "Cooldown" states
   *  occur.)
   */
  pub light_engine_state: u8,

  /**
   * The playback_state is one of three state machines in the DAC.
   * It reports the state of the playback system.
   *
   * The DAC has one playback system, which buffers data and sends it
   * to the analog output hardware at its current point rate. At any
   * given time, the playback system is connected to a source. Usually,
   * the source is the network streamer, which uses the protocol 
   * described in this document; however, other sources exist, such as
   * a built-in abstract generator and file playback from SD card. The
   * playback system is in one of the following states:
   *
   *   - 0: Idle. This is the default state. No points may be added to
   *        the buffer. No output is generated; all analog outputs are
   *        at 0v, and the shutter is controlled by the data source.
   *   - 1: Prepared. The buffer will accept points. The output is the
   *        same as in the Idle state.
   *   - 2: Playing. Points are being sent to the output.
   *
   * See playback_flags for additional information.
   */
  pub playback_state: u8,

  /**
   * The currently-selected data source is specified in the source field:
   *
   *   - 0: Network streaming (the protocol defined in the rest of this
   *        document).
   *   - 1: ILDA playback from SD card.
   *   - 2: Internal abstract generator.
   */
  pub source: u8,

  /**
   * The light_engine_state field gives the current state of the light
   * engine. If the light engine is Ready, light_engine_flags will be 0.
   * Otherwise, bits in light_engine_flags will be set as follows:
   *
   * [0]: Emergency stop occurred due to E-Stop packet or invalid
   *      command.
   * [1]: Emergency stop occurred due to E-Stop input to projector.
   * [2]: Emergency stop input to projector is currently active.
   * [3]: Emergency stop occurred due to overtemperature condition.
   * [4]: Overtemperature condition is currently active.
   * [5]: Emergency stop occurred due to loss of Ethernet link.
   * [15:5]: Future use.
   */
  pub light_engine_flags: u16,

  /**
   * The playback_flags field may be nonzero during normal operation.
   * Its bits are defined as follows:
   *
   * [0]: Shutter state: 0 = closed, 1 = open.
   * [1]: Underflow. 1 if the last stream ended with underflow, rather
   *      than a Stop command. Reset to zero by the Prepare command.
   * [2]: E-Stop. 1 if the last stream ended because the E-Stop state
   *      was entered. Reset to zero by the Prepare command.
   */
  pub playback_flags: u16,

  /// TODO: Undocumented?
  pub source_flags: u16,

  /** Reports the number of points currently buffered. */
  pub buffer_fullness: u16,

  /**
   * The number of points per second for which the DAC is configured
   * (if Prepared or Playing), or zero if the DAC is idle.
   */
  pub point_rate: u32,

  /**
   * The number of points that the DAC has actually emitted since it
   * started playing (if Playing), or zero (if Prepared or Idle).
   */
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

pub enum Command {
  BeginPlayback,
  ClearEStop,
  EmergencyStop,
  Ping,
  PrepareStream,
  QueueRateChange,
  Stop,
  WriteData,
  UnknownCommand { command: u8 },
}

impl Command {
  /// Returns the over-the-wire serialization of the command
  pub fn value(&self) -> u8 {
    match *self {
      Command::BeginPlayback => 0x62,   // 'b'
      Command::ClearEStop => 0x63,      // 'c'
      Command::EmergencyStop=> 0x00,    // also recognizes 0xff
      Command::Ping => 0x3f,            // '?'
      Command::PrepareStream => 0x70,   // 'p'
      Command::QueueRateChange => 0x74, // 'q'
      Command::Stop => 0x73,            // 's'
      Command::WriteData => 0x64,       // 'd'
      Command::UnknownCommand { command } => command,
    }
  }
}

struct BeginCommand {
  command: u8, // 'b' (0x62)
  low_water_mark: u16, // currently unused.
  point_rate: u32,
}

struct PrepareCommand {
  command: u8, // 'd' (0x64)
  num_points: u16,
  dac_points: Vec<Point>,
}

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

// TODO BETTER NAME
pub enum ResponseState {
  Ack,
  BufferFull,
  InvalidCommand,
  Stop,
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
  response: ResponseState,

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
  pub fn new(response: ResponseState, command: u8, dac_status: DacStatus) 
      -> DacResponse {
    DacResponse {
      response: response,
      command: command,
      dac_status: dac_status,
    }
  }

  pub fn info() -> DacResponse {
    DacResponse {
      response: ResponseState::Ack, //0x61,
      command: 0x3f, // '?'
      dac_status: DacStatus::empty(),
    }
  }

  pub fn serialize(&self) -> Vec<u8> {
    let mut vec = Vec::new();
    let response = match self.response {
      ResponseState::Ack => 0x61,
      ResponseState::BufferFull => 0x46,
      ResponseState::InvalidCommand => 0x49,
      ResponseState::Stop => 0x21,
    };
    vec.push(response);
    vec.push(self.command);
    vec.extend(self.dac_status.serialize());
    vec
  }
}

