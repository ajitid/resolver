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
  
  pub fn push_gutter(&mut self, width: usize, height: usize) {
    let mut s = String::new();
    for _ in 0..height {
      s.push_str(&format!("{}│\r\n", " ".repeat(width-1)));
    }
    self.push_col(width, &s);
  }
  
  pub fn push_divider(&mut self, height: usize) {
    let mut s = String::new();
    for _ in 0..height {
      s.push_str("│\r\n");
    }
    self.push_col(1, &s);
  }
  
  pub fn push_col(&mut self, width: usize, text: &str) {
    for (i, l) in text.lines().enumerate() {
      let n = l.len(); // TODO: unicode support
      let l = if n < width {
        format!("{}{}", l, " ".repeat(width - n))
      }else{
        l.to_string()
      };
      if self.data.len() > i {
        self.data[i].push_str(&l);
      }else{
        self.data.push(l);
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
