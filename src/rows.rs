use std::io;
use std::io::stdout;
use std::io::Write;
use std::str;

use crate::buffer::Buffer;

pub struct Rows {
  width: usize,
  text: String,
}

impl Rows {
  pub fn new(width: usize, text: String) -> Self {
    Rows{
      width: width,
      text: text,
    }
  }
  
  pub fn lines<'a>(&'a self) -> Vec<&'a str> {
    let mut b = Vec::new();
    for l in self.text.lines() {
      
    }
    b
  }
  
  pub fn text<'a>(&'a self) -> &'a str {
    &self.text
  }
}

