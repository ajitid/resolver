use std::time;
use std::io::{self, Write};
use std::io::stdout;
use std::cmp::min;

use crossterm;
use crossterm::event;
use crossterm::queue;
use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal;

use clap::Parser;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
  #[clap(long)]
  debug: bool,
  #[clap(long)]
  verbose: bool,
}

struct Finalize {
  opts: Options,
}

impl Drop for Finalize {
  fn drop(&mut self) {
    terminal::disable_raw_mode().expect("Could not finalize terminal (good luck)");
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

struct Editor {
  reader: Reader,
  writer: Writer,
  cursor: Cursor,
  content: Content,
}

impl Editor {
  fn new() -> Self {
    let size = terminal::size().unwrap();
    Editor{
      reader: Reader,
      writer: Writer::new(size),
      cursor: Cursor::new(size),
      content: Content::new(size.0 as usize),
    }
  }
  
  fn key(&mut self) -> crossterm::Result<bool> {
    match self.reader.read_key()? {
      event::KeyEvent{
        code: event::KeyCode::Char('d'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => return Ok(false),
      event::KeyEvent{
        code: v @ (event::KeyCode::Up | event::KeyCode::Down | event::KeyCode::Left | event::KeyCode::Right | event::KeyCode::Home | event::KeyCode::End),
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.cursor.move_dir(v),
      event::KeyEvent{
        code: event::KeyCode::Char(v),
        modifiers: event::KeyModifiers::NONE | event::KeyModifiers::SHIFT,
        ..
      } => self.cursor.move_rel(self.content.insert_rel(v)),
      event::KeyEvent{
        code: v @ event::KeyCode::Enter,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.cursor.move_rel(self.content.insert_ctl(v)),
      _ => {},
    };
    Ok(true)
  }
  
  fn draw(&mut self) -> crossterm::Result<bool> {
    self.writer.refresh(&self.cursor, &self.content)?;
    Ok(true)
  }
  
  fn step(&mut self) -> crossterm::Result<bool> {
    let res = self.key()?;
    self.draw()?;
    Ok(res)
  }
}

struct Buffer {
  data: String,
}

impl Buffer {
  fn new() -> Self {
    Buffer{
      data: String::new(),
    }
  }
  
  fn push(&mut self, c: char) {
    self.data.push(c);
  }
  
  fn push_str(&mut self, s: &str) {
    self.data.push_str(s);
  }
  
  fn content(&mut self, c: &Content) {
    self.data.push_str(&c.text);
  }
  
  fn clear(&mut self) {
    self.data.clear();
  }
}

impl io::Write for Buffer {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match std::str::from_utf8(buf) {
      Ok(v) => {
        self.push_str(v);
        Ok(v.len())
      },
      Err(_) => Err(io::ErrorKind::WriteZero.into()),
    }
  }
  
  fn flush(&mut self) -> io::Result<()> {
    let out = write!(stdout(), "{}", self.data);
    stdout().flush()?;
    self.data.clear();
    out
  }
}

struct Cursor {
  x: u16,
  y: u16,
  size: (u16, u16),
}

impl Cursor {
  fn new(size: (u16, u16)) -> Cursor {
    Cursor{
      x: 0,
      y: 0,
      size: size,
    }
  }
  
  fn move_dir(&mut self, dir: event::KeyCode) {
    match dir {
      event::KeyCode::Left  => self.x = if self.x > 0 { self.x - 1 } else { 0 },
      event::KeyCode::Down  => self.y = min(self.size.1 - 1, self.y + 1),
      event::KeyCode::Up    => self.y = if self.y > 0 { self.y - 1 } else { 0 },
      event::KeyCode::Right => self.x = min(self.size.0 - 1, self.x + 1),
      event::KeyCode::Home  => self.x = 0,
      event::KeyCode::End   => self.x = self.size.0,
      _ => unimplemented!(),
    }
  }

  fn move_rel(&mut self, mvt: Movement) {
    let (x, y) = mvt.apply((self.x as usize, self.y as usize));
    self.x = x as u16;
    self.y = y as u16;
  }
}

struct Stride {
  n: usize,
  abs: bool,
}

impl Stride {
  fn abs(v: usize) -> Stride {
    Stride{
      n: v,
      abs: true,
    }
  }
  
  fn rel(v: usize) -> Stride {
    Stride{
      n: v,
      abs: false,
    }
  }
  
  fn apply(&self, v: usize) -> usize {
    if self.abs {
      self.n
    }else{
      self.n + v
    }
  }
}

struct Movement {
  x: Stride,
  y: Stride,
}

impl Movement {
  fn apply(&self, c: (usize, usize)) -> (usize, usize) {
    (self.x.apply(c.0), self.y.apply(c.1))
  }
}

struct Content {
  text: String,
  width: usize,
  cursor: usize,
  lines: Vec<String>,
}

impl Content {
  fn new(width: usize) -> Content {
    Content{
      text: String::new(),
      width: width,
      cursor: 0,
      lines: Vec::new(),
    }
  }
  
  fn insert(&mut self, idx: usize, c: char) -> Movement {
    let l = self.text.len();
    let idx = if idx > l { l } else { idx };
    self.text.insert(idx, c);
    if c == '\n' {
      Movement{x: Stride::abs(0), y: Stride::rel(1)}
    }else{
      Movement{x: Stride::rel(1), y: Stride::rel(0)}
    }
  }
  
  fn insert_rel(&mut self, c: char) -> Movement {
    self.cursor += 1;
    self.insert(self.cursor, c)
  }
  
  fn insert_ctl(&mut self, k: event::KeyCode) -> Movement {
    match k {
      event::KeyCode::Enter => self.insert_rel('\n'),
      _ => unimplemented!(),
    }
  }
  
  fn insert_str(&mut self, idx: usize, text: &str) {
    let l = self.text.len();
    let idx = if idx > l { l } else { idx };
    self.text.insert_str(idx, text);
  }
}

struct Writer {
  term_size: (u16, u16),
  buffer: Buffer,
}

impl Writer {
  fn new(size: (u16, u16)) -> Self {
    Self{
      term_size: size,
      buffer: Buffer::new(),
    }
  }
  
  fn clear() -> crossterm::Result<()> {
    execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
    Self::move_cursor(0, 0)?;
    Ok(())
  }
  
  fn move_cursor(x: u16, y: u16) -> crossterm::Result<()> {
    execute!(stdout(), cursor::MoveTo(x, y))
  }
  
  fn draw(&mut self, _: &Content) -> crossterm::Result<()> {
    let rows = self.term_size.1;
    for i in 0..rows {
      self.buffer.push('~');
      if i == 2 {
        self.buffer.push_str(" RESOLVER. The 'Soulver' in your terminal.");
      }else if i == 3 {
        self.buffer.push_str(&format!(" v{}", VERSION));
      }
      if i < rows - 1 {
        self.buffer.push_str("\r\n");
      }
    }
    Ok(())
  }
  
  fn refresh(&mut self, cursor: &Cursor, content: &Content) -> crossterm::Result<()> {
    queue!(self.buffer, cursor::Hide, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;
    // self.draw(content)?;
    self.buffer.content(content);
    queue!(self.buffer, cursor::MoveTo(cursor.x, cursor.y), cursor::Show)?;
    self.buffer.flush()
  }
}

fn main() -> crossterm::Result<()> {
  let opts = Options::parse();
  let _cleanup = Finalize{opts: opts.clone()};
  terminal::enable_raw_mode()?;
  
  let mut editor = Editor::new();
  editor.draw()?;
  
  loop {
    if !editor.step()? {
      break;
    }
  }
  
  Ok(())
}
