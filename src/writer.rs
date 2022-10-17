use std::io::stdout;
use std::io::Write;

use crossterm;
use crossterm::queue;
use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal;

use crate::buffer::Buffer;
use crate::text::{Text, Pos};
use crate::frame::Frame;

use crate::rdl::scan;
use crate::rdl::parse;
use crate::rdl::exec;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Writer {
  term_size: (usize, usize),
  text_size: (usize, usize),
  frame: Frame,
  buf: Buffer,
  doc: Buffer,
}

impl Writer {
  pub fn new_with_size(size: (usize, usize)) -> Self {
    Self{
      term_size: size,
      text_size: ((size.0 / 3) * 2, size.1 - 1),
      frame: Frame::new(size.0),
      buf: Buffer::new(),
      doc: Buffer::new(),
    }
  }
  
  pub fn clear() -> crossterm::Result<()> {
    execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
    execute!(stdout(), cursor::MoveTo(0, 0))?;
    Ok(())
  }
  
  fn draw(&mut self) -> crossterm::Result<()> {
    let rows = self.term_size.1;
    for i in 0..rows {
      self.doc.push('~');
      if i == 2 {
        self.doc.push_str(" RESOLVER. The 'Soulver' in your terminal.");
      }else if i == 3 {
        self.doc.push_str(&format!(" v{}", VERSION));
      }
      if i < rows - 1 {
        self.doc.push_str("\r\n");
      }
    }
    Ok(())
  }
  
  fn draw_welcome(width: usize, height: usize) -> String {
    let mut g = String::new();
    for i in 0..height {
      g.push('~');
      if i == 2 {
        g.push_str(" RESOLVER. The 'Soulver' in your terminal.");
      }else if i == 3 {
        g.push_str(&format!(" v{}", VERSION));
      }else if i == 5 {
        g.push_str(" Cool formula output will go on this side. Eventually.");
      }
      g.push('\n');
    }
    g
  }
  
  fn draw_formula(width: usize, height: usize, text: &Text) -> String {
    let mut g = String::new();
    let mut cxt = exec::Context::new();

    for l in text.lines() {
      let mut p = parse::Parser::new(scan::Scanner::new(l));
      let mut i = 0;
      loop {
        let root = match p.parse() {
          Ok(root) => root,
          Err(_)  => break,
        };
        
        if i > 0 { g.push_str("; "); }
        g.push_str(&format!("{}", root));

        let res = match root.exec(&cxt) {
          Ok(res) => res,
          Err(_)  => continue,
        };
        g.push_str(&format!(" → {}", res));
        
        i += 1;
      }
    }
    
    g
  }
  
  fn draw_gutter(width: usize, height: usize) -> String {
    let mut g = String::new();
    for i in 0..height {
      g.push_str(&format!(" {:>3}\n", i+1));
    }
    g
  }
  
  pub fn refresh(&mut self, pos: &Pos, text: &Text) -> crossterm::Result<()> {
    queue!(self.buf, cursor::Hide, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;
    let tw = (self.term_size.0 / 3) - 6;
    let gw = 5;
    
    let gutter = Text::new_with_string(gw, Writer::draw_gutter(gw, self.term_size.1 as usize));
    let ticker = Text::new_with_string(tw, Writer::draw_formula(tw, self.term_size.1 as usize, text));
    let cols = vec![&gutter, &text, &ticker];
    self.frame.write_cols(cols, self.term_size.1 as usize, &mut self.buf);
    
    queue!(self.buf, cursor::MoveTo((pos.x + gw + 1) as u16, pos.y as u16), cursor::Show)?;
    self.buf.flush()
  }
}
