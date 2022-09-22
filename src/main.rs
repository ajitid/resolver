use std::time;
use std::io::stdout;

use crossterm;
use crossterm::event;
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
  
  fn keystroke(&self) -> crossterm::Result<bool> {
    match self.reader.read_key()? {
      event::KeyEvent{
        code: event::KeyCode::Char('d'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => Ok(false),
      _ => Ok(true),
    }
  }
  
  fn step(&self) -> crossterm::Result<bool> {
    self.writer.refresh()?;
    self.keystroke()
  }
}

struct Writer {
  term_size: (usize, usize),
}

impl Writer {
  fn new() -> Self {
    let size = terminal::size().map(|(x, y)| (x as usize, y as usize)).unwrap();
    Self{
      term_size: size,
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
  
  fn draw_gutter(&self) -> crossterm::Result<()> {
    let rows = self.term_size.1;
    for i in 0..rows {
      print!("~");
      if i < rows - 1 {
        println!("\r");
      }
    }
    Ok(())
  }
  
  fn refresh(&self) -> crossterm::Result<()> {
    Self::clear()?;
    self.draw_gutter()?;
    Self::move_cursor(0, 0)?;
    Ok(())
  }
}

fn main() -> crossterm::Result<()> {
  let _cleanup = Finalize{};
  terminal::enable_raw_mode()?;
  
  let editor = Editor::new();
  loop {
    if !editor.step()? {
      break;
    }
  }
  
  Ok(())
}
