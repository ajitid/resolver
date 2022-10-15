use std::fmt;
use std::ops;
use std::error;
use std::num::ParseFloatError;

#[derive(Debug, Eq, PartialEq)]
pub struct IOError {
  msg: String,
}

impl IOError {
  pub fn new(msg: &str) -> IOError {
    IOError{
      msg: msg.to_string(),
    }
  }
}

impl error::Error for IOError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}

impl fmt::Display for IOError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "I/O error: {}", self.msg)
  }
}

#[derive(Debug, Eq, PartialEq)]
pub struct AssertionFailed {
  msg: Option<String>,
}

impl AssertionFailed {
  pub fn new() -> AssertionFailed {
    AssertionFailed{
      msg: None,
    }
  }

  pub fn new_with_message(msg: &str) -> AssertionFailed {
    AssertionFailed{
      msg: Some(msg.to_string()),
    }
  }
}

impl error::Error for AssertionFailed {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}

impl fmt::Display for AssertionFailed {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(msg) = &self.msg {
      write!(f, "Assertion failed: {}", msg)
    }else{
      write!(f, "Assertion failed")
    }
  }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SyntaxError {
  src: String,
  loc: ops::Range<usize>,
  msg: String,
}

impl SyntaxError {
  pub fn new(s: &str, l: ops::Range<usize>, m: &str) -> SyntaxError {
    SyntaxError{
      src: s.to_owned(),
      loc: l,
      msg: m.to_string(),
    }
  }
}

impl error::Error for SyntaxError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}

impl fmt::Display for SyntaxError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Syntax error: {}", self.msg)
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
  IOError(IOError),
  EndOfInput,
  TokenNotMatched,
  UnboundVariable(String),
  AssertionFailed(AssertionFailed),
  SyntaxError(SyntaxError),
  ParseFloatError(ParseFloatError),
}

impl From<IOError> for Error {
  fn from(error: IOError) -> Self {
    Self::IOError(error)
  }
}

impl From<AssertionFailed> for Error {
  fn from(error: AssertionFailed) -> Self {
    Self::AssertionFailed(error)
  }
}

impl From<SyntaxError> for Error {
  fn from(error: SyntaxError) -> Self {
    Self::SyntaxError(error)
  }
}

impl From<ParseFloatError> for Error {
  fn from(error: ParseFloatError) -> Self {
    Self::ParseFloatError(error)
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::IOError(err) => err.fmt(f),
      Self::EndOfInput => write!(f, "Unexpected end of input"),
      Self::TokenNotMatched => write!(f, "Token not matched"),
      Self::UnboundVariable(name) => write!(f, "No such variable: {}", name),
      Self::AssertionFailed(err) => err.fmt(f),
      Self::SyntaxError(err) => err.fmt(f),
      Self::ParseFloatError(err) => err.fmt(f),
    }
  }
}
