use crossterm::queue;
use crossterm::style;
use crossterm::cursor;
use crossterm::terminal;

use crate::error;
use crate::text::{Renderable, Pos};
use crate::text::attrs;
use crate::buffer::Buffer;
use crate::options;

pub struct Frame {
  opts: options::Options,
  clear_on_write: bool,
  _width: usize,
  sep: char,
}

impl Frame {
  pub fn new(width: usize, opts: options::Options) -> Self {
    Frame{
      opts: opts,
      clear_on_write: true,
      _width: width,
      sep: '┊',
    }
  }
  
  pub fn write_cols(&self, cols: Vec<&dyn Renderable>, height: usize, buf: &mut Buffer, vpos: &Pos) -> Result<usize, error::Error> {
    let highlight: attrs::Attributes = attrs::Attributes{
      bold: false, invert: false, color: None, background: Some(style::Color::Rgb{r: 10, g: 10, b: 10}),
    };
    let lines: Vec<usize> = cols.iter().map(|t| { t.num_lines() }).collect();
    let lmax: usize = match lines.iter().reduce(|a, b| {
      if a > b { a } else { b }
    }) {
      Some(v) => *v,
      None => return Ok(0), // nothing to do
    };
    for i in 0..lmax { // until all content is consumed
      if self.clear_on_write {
        queue!(buf, cursor::MoveTo(0, i as u16), terminal::Clear(terminal::ClearType::CurrentLine))?;
      }
      for (x, c) in cols.iter().enumerate() {
        let adj = if x > 0 {
          buf.push(self.sep);
          1
        }else{
          0
        };
        let (n, b) = if i == vpos.y {
          let hspan = vec![attrs::Span::new(0..c.width(), highlight.clone())];
          let attrs = match c.attributes() {
            Some(v) => attrs::merge(v.clone(), hspan),
            None => hspan,
          };
          c.write_line_with_attrs(i, buf, Some(&attrs))
        }else{
          c.write_line(i, buf)
        };
        if self.opts.debug_editor {
          buf.push_str(&format!(" {} {} ({}, {})", n, b, vpos.x, vpos.y));
        }
        let w = c.width();
        if n < w - adj {
          let pad = " ".repeat(w - adj - n);
          if i == vpos.y {
            buf.push_str(&attrs::render(&pad, &vec![attrs::Span::new(0..c.width(), highlight.clone())]));
          }else{
            buf.push_str(&pad);
          }
        }
      }
      if i < height - 1 {
        buf.push_str("\r\n");
      }
    }
    Ok(lmax)
  }
}
