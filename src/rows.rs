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
  
  pub fn push_gutter(&mut self, _w: usize, h: usize) {
    let mut s = String::new();
    for _ in 0..h {
      s.push_str("   │\r\n");
    }
    self.push_col(&s);
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
  
  pub fn render(&self) -> String {
    let mut s = String::new();
    for l in &self.data {
      s.push_str(&l);
      s.push_str("\r\n");
    }
    s
  }
  
  pub fn clear(&mut self) {
    self.data.clear();
  }
}
