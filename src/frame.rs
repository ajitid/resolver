use std::io;
use std::io::stdout;
use std::io::Write;

use crate::buffer::Buffer;

pub struct Frame {
  cols: Vec<Rows>,
}

impl Frame {
  pub fn new() -> Self {
    Frame{
      cols: Vec::new(),
    }
  }
  
  pub fn push_col(&mut self, col: Rows) {
    self.cols.push(col)
  }
  
  pub fn render(&self) -> String {
    let mut s = String::new();
    for l in &self.cols {
      s.push_str(&l);
      s.push_str("\r\n");
    }
    s
  }
  
  pub fn clear(&mut self) {
    self.cols.clear();
  }
}

impl io::Write for Buffer {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match std::str::from_utf8(buf) {
      Ok(v) => {
        self.push_str(v);
        Ok(v.len())
      },
      Err(_) => Err(io::ErrorKind::WriteZero.into()),
    }
  }
  
  fn flush(&mut self) -> io::Result<()> {
    let out = write!(stdout(), "{}", self.data);
    stdout().flush()?;
    self.data.clear();
    out
  }
}
