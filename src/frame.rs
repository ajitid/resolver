use std::io;
use std::io::stdout;
use std::io::Write;

use crate::text::Text;
use crate::buffer::Buffer;

pub struct Frame {
  width: usize,
  sep: char,
}

impl Frame {
  pub fn new(width: usize) -> Self {
    Frame{
      width: width,
      sep: '┊',
    }
  }
  
  pub fn write_cols(&self, cols: Vec<&Text>, buf: &mut Buffer) -> usize {
    let mut i = 0;
    let lines: Vec<usize> = cols.iter().map(|t| { t.num_lines() }).collect();
    let lmax: usize = match cols.iter().map(|t| { t.num_lines() }).reduce(|a, b| {
      if a > b { a } else { b }
    }) {
      Some(v) => v,
      None => return 0, // nothing to do
    };
    for i in 0..=lmax { // until all content is consumed
      for (x, c) in cols.iter().enumerate() {
        if x > 0 {
          buf.push(self.sep);
        }
        let n = c.write_line(i, buf);
        let w = c.width();
        if n < w {
          buf.push_str(&" ".repeat(w - n));
        }
      }
      buf.push_str("\r\n");
    }
    lmax
  }
}
