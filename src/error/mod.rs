use std::io;
use std::fmt;
use std::string;
use std::error;

#[derive(Debug)]
pub enum Error {
  IOError(io::Error),
  UTF8Error(string::FromUtf8Error),
}

impl From<io::Error> for Error {
  fn from(error: io::Error) -> Self {
    Self::IOError(error)
  }
}

impl From<string::FromUtf8Error> for Error {
  fn from(error: string::FromUtf8Error) -> Self {
    Self::UTF8Error(error)
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::IOError(err) => err.fmt(f),
      Self::UTF8Error(err) => err.fmt(f),
    }
  }
}
