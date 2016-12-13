// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use std::error;
use std::io;
use std::fmt;
use std::convert::From;
use std::sync::PoisonError;

/// System-wide error type.
#[derive(Debug)]
pub enum EmulatorError {
  /// Miscellaneous client error.
  ClientError,
  /// Network error.
  IoError { cause: io::Error },
  /// An issue obtaining a std::sync lock. Should not occur.
  LockError,
  /// Error parsing client request.
  ParseError,
  /// Cannot put anything else on the point pipeline.
  PipelineFull,
  /// Unknown command received from the client. Some client commands are valid,
  /// but we do not yet support them.
  UnknownCommand,
}

impl fmt::Display for EmulatorError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let error_detail = match *self {
      EmulatorError::IoError { ref cause  } => {
        return write!(f, "IoError {}", cause);
      },
      EmulatorError::ClientError => "ClientError",
      EmulatorError::LockError => "LockError",
      EmulatorError::ParseError => "ParseError",
      EmulatorError::PipelineFull => "PipelineFull",
      EmulatorError::UnknownCommand => "UnknownCommand",
    };
    write!(f, "{}", error_detail)
  }
}

impl error::Error for EmulatorError {
  fn description(&self) -> &str {
    match *self {
      EmulatorError::ClientError => "ClientError",
      EmulatorError::IoError { .. } => "IoError",
      EmulatorError::LockError => "LockError",
      EmulatorError::ParseError => "ParseError",
      EmulatorError::PipelineFull => "PipelineFull",
      EmulatorError::UnknownCommand => "UnknownCommand",
    }
  }
}

impl From<io::Error> for EmulatorError {
  fn from(error: io::Error) -> EmulatorError {
    EmulatorError::IoError { cause: error }
  }
}

impl<T> From<PoisonError<T>> for EmulatorError {
  fn from(_error: PoisonError<T>) -> EmulatorError {
    EmulatorError::LockError
  }
}
