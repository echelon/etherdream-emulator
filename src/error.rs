// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use std::error;
use std::io;
use std::fmt;
use std::convert::From;

/// Represents an error that occurred when talking to the client.
#[derive(Debug)]
pub enum ClientError {
  ConnectionError,
  ParseError,
}

impl fmt::Display for ClientError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "An error lol k")
  }
}

impl error::Error for ClientError {
  fn description(&self) -> &str {
    /*match self {
      ConnectionError => "There was a problem with the client connection.",
      ParseError => "There was a problem parsing the client protocol.",
    }*/
    "foo"
  }
}

impl From<io::Error> for ClientError {
  fn from(error: io::Error) -> ClientError {
    ClientError::ConnectionError
  }
}
