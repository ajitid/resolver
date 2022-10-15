use std::io;
use std::fmt;
use std::string;
use std::error;

#[derive(Debug)]
pub struct Generic {
  msg: String,
}

impl Generic {
  pub fn new(msg: String) -> Generic {
    Generic{msg: msg}
  }
}

impl error::Error for Generic {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}

impl fmt::Display for Generic {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.msg)
  }
}

#[derive(Debug)]
pub enum Error {
  Generic(Generic),
  IOError(io::Error),
  UTF8Error(string::FromUtf8Error),
}

impl From<Generic> for Error {
  fn from(error: Generic) -> Self {
    Self::Generic(error)
  }
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
      Self::Generic(err) => err.fmt(f),
      Self::IOError(err) => err.fmt(f),
      Self::UTF8Error(err) => err.fmt(f),
    }
  }
}
