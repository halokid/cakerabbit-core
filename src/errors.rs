use rmpv::decode;
use std::{error, fmt, io};

#[derive(Debug)]
pub struct CakeError(pub String);

#[derive(Debug)]
pub enum DecodeError {
  Truncated,
  Invalid,
  UnknownIo(io::Error),
}

impl DecodeError {
  fn description(&self) -> &str {
    match *self {
      DecodeError::Truncated => "could not read enough bytes to decode a complete message",
      DecodeError::UnknownIo(_) => "Unknown IO error while decoding a message",
      DecodeError::Invalid => "the byte sequence is not a valid msgpack-rpc message",
    }
  }
}

impl fmt::Display for DecodeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    self.description().fmt(f)
  }
}

impl error::Error for DecodeError {
  fn description(&self) -> &str {
    self.description()
  }

  fn cause(&self) -> Option<&dyn error::Error> {
    match *self {
      DecodeError::UnknownIo(ref e) => Some(e),
      _ => None,
    }
  }
}

impl From<io::Error> for DecodeError {
  fn from(err: io::Error) -> DecodeError {
    match err.kind() {
      io::ErrorKind::UnexpectedEof => DecodeError::Truncated,
      io::ErrorKind::Other => {
        if let Some(cause) = err.get_ref().unwrap().source() {
          if cause.to_string() == "type mismatch" {
            return DecodeError::Invalid;
          }
        }
        DecodeError::UnknownIo(err)
      }
      _ => DecodeError::UnknownIo(err),
    }
  }
}

impl From<decode::Error> for DecodeError {
  fn from(err: decode::Error) -> DecodeError {
    match err {
      decode::Error::InvalidMarkerRead(io_err) | decode::Error::InvalidDataRead(io_err) => {
        From::from(io_err)
      }
    }
  }
}
