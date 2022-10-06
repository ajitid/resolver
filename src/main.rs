mod buffer;
mod content;
mod editor;
mod rows;

use std::time;
use std::io::stdout;
use std::io::Write;
use std::fs;

use crossterm;
use crossterm::event;
use crossterm::queue;
use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal;

use clap::Parser;

use buffer::Buffer;
use content::{Content, Pos};
use editor::Editor;
use rows::Rows;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
  #[clap(long)]
  debug: bool,
  #[clap(long)]
  verbose: bool,
  #[clap(help="Document to open")]
  doc: Option<String>,
}

struct Finalize {
  opts: Options,
}

impl Drop for Finalize {
  fn drop(&mut self) {
    terminal::disable_raw_mode().expect("Could not finalize terminal (good luck)");
    execute!(stdout(), terminal::LeaveAlternateScreen).expect("Could not exit alternate screen");
    if !self.opts.debug {
      Writer::clear().expect("Could not clear screen");
    }
  }
}

struct Reader;

impl Reader {
  fn read_key(&self) -> crossterm::Result<event::KeyEvent> {
    loop {
      if event::poll(time::Duration::from_millis(500))? {
        if let event::Event::Key(event) = event::read()? {
          return Ok(event);
        }
      }
    }
  }
}

struct Cursor {
  x: u16,
  y: u16,
}

impl Cursor {
  fn new(_size: (u16, u16)) -> Cursor {
    Cursor{
      x: 0,
      y: 0,
    }
  }
  
  fn move_abs(&mut self, pos: Pos) {
    self.x = pos.x as u16;
    self.y = pos.y as u16;
  }
}

struct Writer {
  term_size: (u16, u16),
  text_size: (u16, u16),
  buf: Buffer,
  doc: Buffer,
  rows: Rows,
}

impl Writer {
  fn new(size: (u16, u16)) -> Self {
    Self{
      term_size: size,
      text_size: (size.0 / 3 * 2, size.1 - 1),
      buf: Buffer::new(),
      doc: Buffer::new(),
      rows: Rows::new(),
    }
  }
  
  fn clear() -> crossterm::Result<()> {
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
  
  fn refresh(&mut self, cursor: &Cursor, content: &Content) -> crossterm::Result<()> {
    queue!(self.buf, cursor::Hide, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;
    if content.len() > 0 {
      content.fill(&mut self.doc);
    }else{
      self.draw()?;
    }
    let gw = 3;
    self.rows.push_gutter(gw as usize, (self.term_size.1 - 1) as usize);
    self.rows.push_col(self.text_size.0 as usize, self.doc.text());
    self.rows.push_divider((self.term_size.1 - 1) as usize);
    self.buf.push_rows(&self.rows);
    queue!(self.buf, cursor::MoveTo(cursor.x + ((gw + 1) as u16), cursor.y), cursor::Show)?;
    self.doc.clear();
    self.rows.clear();
    self.buf.flush()
  }
}

fn main() -> crossterm::Result<()> {
  let opts = Options::parse();
  let _cleanup = Finalize{opts: opts.clone()};
  execute!(stdout(), terminal::EnterAlternateScreen)?;
  terminal::enable_raw_mode()?;
  
  let mut editor = Editor::new();
  if let Some(doc) = opts.doc {
    match fs::read_to_string(doc) {
      Ok(text) => editor.set_text(text),
      Err(err) => return Err(err.into()),
    };
  }
  
  editor.draw()?;
  loop {
    if !editor.step()? {
      break;
    }
  }
  
  Ok(())
}
