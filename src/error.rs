// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use std::error;
use std::io;
use std::fmt;
use std::convert::From;
use std::sync::PoisonError;

/// System-wide error type.
#[derive(Debug)]
pub enum EmulatorError {
  /// Network error.
  IoError { cause: io::Error },
  /// An issue obtaining a std::sync lock. Should not occur.
  LockError,
  /// Cannot put anything else on the point pipeline.
  PipelineFull,
  /// Unknown command received from the client. Some client commands are valid,
  /// but we do not yet support them.
  UnknownCommand,
}

// TODO: Unify error types.
/// Represents an error that occurred when talking to the client.
#[derive(Debug)]
pub enum ClientError {
  ConnectionError,
  ParseError,
}

impl fmt::Display for ClientError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let error_detail = match *self {
      ClientError::ConnectionError => "ConnectionError",
      ClientError::ParseError => "ParseError",
    };
    write!(f, "{}", error_detail)
  }
}

impl error::Error for ClientError {
  fn description(&self) -> &str {
    match *self {
      ClientError::ConnectionError =>
          "There was a problem with the client connection.",
      ClientError::ParseError =>
          "There was a problem parsing the client protocol.",
    }
  }
}

impl From<io::Error> for ClientError {
  fn from(_error: io::Error) -> ClientError {
    ClientError::ConnectionError
  }
}

impl error::Error for EmulatorError {
  fn description(&self) -> &str {
    "TODO" // TODO
  }
}

impl From<io::Error> for EmulatorError {
  fn from(error: io::Error) -> EmulatorError {
    EmulatorError::IoError { cause: error }
  }
}

impl fmt::Display for EmulatorError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "TODO") // TODO
  }
}

impl<T> From<PoisonError<T>> for EmulatorError {
  fn from(_error: PoisonError<T>) -> EmulatorError {
    EmulatorError::LockError
  }
}
