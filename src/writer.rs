use std::io::stdout;
use std::io::Write;

use crossterm;
use crossterm::queue;
use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal;
use crossterm::style::Color;

use crate::options;
use crate::buffer::Buffer;
use crate::text::{Text, Content, Storage, Renderable, Pos};
use crate::text::attrs;
use crate::frame::Frame;

use crate::rdl;
use crate::rdl::exec;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Writer {
  opts: options::Options,
  term_size: (usize, usize),
  text_size: (usize, usize),
  frame: Frame,
  buf: Buffer,
}

impl Writer {
  pub fn new_with_size(size: (usize, usize), opts: options::Options) -> Self {
    Self{
      opts: opts.clone(),
      term_size: size,
      text_size: ((size.0 / 3) * 2, size.1 - 1),
      frame: Frame::new(size.0, opts),
      buf: Buffer::new(),
    }
  }
  
  pub fn clear() -> crossterm::Result<()> {
    execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
    execute!(stdout(), cursor::MoveTo(0, 0))?;
    Ok(())
  }
  
  fn _draw_welcome(_width: usize, height: usize) -> String {
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
  
  fn draw_formula(width: usize, _height: usize, text: &Text) -> (Content, Content) {
    let mut edit_text = String::new();
    let mut edit_spns: Vec<attrs::Span> = Vec::new();
    let mut fmla_text = String::new();
    let mut fmla_spns: Vec<attrs::Span> = Vec::new();
    let mut cxt = exec::Context::new_with_stdlib();
    
    let style = vec![
      attrs::Attributes{bold: true, color: Some(Color::Yellow)},
      attrs::Attributes{bold: true, color: Some(Color::Magenta)},
      attrs::Attributes{bold: true, color: Some(Color::Cyan)},
      attrs::Attributes{bold: true, color: Some(Color::Green)},
      attrs::Attributes{bold: true, color: Some(Color::Blue)},
    ];
    
    let mut boff0 = 0;
    for l in text.lines() {
      let (mut txt, mut exp) = rdl::render_with_attrs(&mut cxt, l, boff0, fmla_text.len(), Some(&style));
      
      edit_text.push_str(txt.text());
      edit_text.push_str("\n");
      edit_spns.append(txt.spans_mut());
      
      fmla_text.push_str(exp.text());
      fmla_text.push_str("\n");
      fmla_spns.append(exp.spans_mut());
      
      boff0 += txt.len() + 1 /* newline */;
    }
    
    (
      Content::new_with_attributed(edit_text, edit_spns, text.width()),
      Content::new_with_attributed(fmla_text, fmla_spns, width)
    )
  }
  
  fn draw_gutter(_width: usize, height: usize) -> String {
    let mut g = String::new();
    for i in 0..height {
      g.push_str(&format!(" {:>3}\n", i+1));
    }
    g
  }
  
  pub fn refresh(&mut self, pos: &Pos, text: &Text) -> crossterm::Result<()> {
    queue!(self.buf, cursor::Hide, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;
    let tw = (self.term_size.0 / 3) - 6;
    let gw = if self.opts.debug_editor { 0 }else{ 5 };
    let ox = if self.opts.debug_editor { 0 }else{ gw + 1 };
    
    let (edit, fmla) = Writer::draw_formula(tw, self.term_size.1 as usize, text);
    let gutter = Content::new_with_string(Writer::draw_gutter(gw, self.term_size.1 as usize), gw);
    let cols: Vec<&dyn Renderable> = if self.opts.debug_editor {
      vec![&edit]
    }else{
      vec![&gutter, &edit, &fmla]
    };
    
    self.frame.write_cols(cols, self.term_size.1 as usize, &mut self.buf, pos);
    queue!(self.buf, cursor::MoveTo((pos.x + ox) as u16, pos.y as u16), cursor::Show)?;
    self.buf.flush()
  }
}
