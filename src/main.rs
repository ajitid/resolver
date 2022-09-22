use std::time;
use std::io::{self, Write};
use std::io::stdout;

use crossterm;
use crossterm::event;
use crossterm::queue;
use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal;

struct Finalize;

impl Drop for Finalize {
  fn drop(&mut self) {
    terminal::disable_raw_mode().expect("Could not finalize terminal (good luck)");
    Writer::clear().expect("Could not clear screen");
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
}

impl Editor {
  fn new() -> Self {
    Editor{
      reader: Reader,
      writer: Writer::new(),
    }
  }
  
  fn keystroke(&mut self) -> crossterm::Result<bool> {
    match self.reader.read_key()? {
      event::KeyEvent{
        code: event::KeyCode::Char('d'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => Ok(false),
      _ => Ok(true),
    }
  }
  
  fn step(&mut self) -> crossterm::Result<bool> {
    self.writer.refresh()?;
    self.keystroke()
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

struct Writer {
  term_size: (usize, usize),
  buffer: Buffer,
}

impl Writer {
  fn new() -> Self {
    let size = terminal::size().map(|(x, y)| (x as usize, y as usize)).unwrap();
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
  
  fn draw_gutter(&mut self) -> crossterm::Result<()> {
    let rows = self.term_size.1;
    for i in 0..rows {
      self.buffer.push('~');
      if i < rows - 1 {
        self.buffer.push_str("\r\n");
      }
    }
    Ok(())
  }
  
  fn refresh(&mut self) -> crossterm::Result<()> {
    queue!(self.buffer, cursor::Hide, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;
    self.draw_gutter()?;
    queue!(self.buffer, cursor::MoveTo(0, 0), cursor::Show)?;
    self.buffer.flush()
  }
}

fn main() -> crossterm::Result<()> {
  let _cleanup = Finalize{};
  terminal::enable_raw_mode()?;
  
  let mut editor = Editor::new();
  loop {
    if !editor.step()? {
      break;
    }
  }
  
  Ok(())
}
