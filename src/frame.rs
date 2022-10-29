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
  
  pub fn write_cols(&self, cols: Vec<&Text>, height: usize, buf: &mut Buffer) -> usize {
    let lines: Vec<usize> = cols.iter().map(|t| { t.num_lines() }).collect();
    let lmax: usize = match lines.iter().reduce(|a, b| {
      if a > b { a } else { b }
    }) {
      Some(v) => *v,
      None => return 0, // nothing to do
    };
    for i in 0..lmax { // until all content is consumed
      for (x, c) in cols.iter().enumerate() {
        let adj = if x > 0 {
          buf.push(self.sep);
          1
        }else{
          0
        };
        let (n, b) = c.write_line(i, buf);
        buf.push_str(&format!(" {} {}", n, b));
        let w = c.width();
        if n < w - adj {
          buf.push_str(&" ".repeat(w - adj - n));
        }
      }
      if i < height - 1 {
        buf.push_str("\r\n");
      }
    }
    lmax
  }
}
