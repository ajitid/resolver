use std::io;
use std::io::stdout;
use std::io::Write;

use crate::buffer::Buffer;

pub struct Rows {
  data: Vec<String>,
}

impl Rows {
  pub fn new() -> Self {
    Rows{
      data: Vec::new(),
    }
  }
  
  pub fn push_col(&mut self, text: &str) {
    for (i, l) in text.lines().enumerate() {
      if self.data.len() > i {
        self.data[i].push_str(l);
      }else{
        self.data.push(l.to_owned());
      }
    }
  }
  
  fn render(&self) -> String {
    let mut s = String::new();
    for l in &self.data {
      s.push_str(&l);
      s.push_str("\n\r");
    }
    s
  }
  
  pub fn clear(&mut self) {
    self.data.clear();
  }
}

impl io::Write for Rows {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match std::str::from_utf8(buf) {
      Ok(v) => {
        self.push_col(v);
        Ok(v.len())
      },
      Err(_) => Err(io::ErrorKind::WriteZero.into()),
    }
  }
  
  fn flush(&mut self) -> io::Result<()> {
    let out = write!(stdout(), "{}", self.render());
    stdout().flush()?;
    self.clear();
    out
  }
}
