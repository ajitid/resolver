use crossterm::event;
use crossterm::terminal;

use crate::Reader;
use crate::Writer;
use crate::Cursor;
use crate::text::Text;

pub struct Editor {
  reader: Reader,
  writer: Writer,
  cursor: Cursor,
  text: Text,
}

impl Editor {
  pub fn new() -> Self {
    let size = terminal::size().unwrap();
    Editor{
      reader: Reader,
      writer: Writer::new(size),
      cursor: Cursor::new(size),
      text: Text::new((size.0 / 3) as usize),
    }
  }
  
  pub fn set_text(&mut self, text: String) {
    self.text.set_text(text)
  }
  
  pub fn key(&mut self) -> crossterm::Result<bool> {
    match self.reader.read_key()? {
      event::KeyEvent{
        code: event::KeyCode::Char('c' | 'd'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => return Ok(false),
      event::KeyEvent{
        code: event::KeyCode::Left,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.cursor.move_abs(self.text.left_rel()),
      event::KeyEvent{
        code: event::KeyCode::Right,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.cursor.move_abs(self.text.right_rel()),
      event::KeyEvent{
        code: event::KeyCode::Up,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.cursor.move_abs(self.text.up_rel()),
      event::KeyEvent{
        code: event::KeyCode::Down,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.cursor.move_abs(self.text.down_rel()),
      event::KeyEvent{
        code: event::KeyCode::Home,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.cursor.move_abs(self.text.home_rel()),
      event::KeyEvent{
        code: event::KeyCode::End,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.cursor.move_abs(self.text.end_rel()),
      event::KeyEvent{
        code: event::KeyCode::Backspace,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.cursor.move_abs(self.text.backspace_rel()),
      event::KeyEvent{
        code: event::KeyCode::Char(v),
        modifiers: event::KeyModifiers::NONE | event::KeyModifiers::SHIFT,
        ..
      } => self.cursor.move_abs(self.text.insert_rel(v)),
      event::KeyEvent{
        code: event::KeyCode::Enter,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.cursor.move_abs(self.text.insert_rel('\n')),
      _ => {},
    };
    Ok(true)
  }
  
  pub fn draw(&mut self) -> crossterm::Result<bool> {
    self.writer.refresh(&self.cursor, &self.text)?;
    Ok(true)
  }
  
  pub fn step(&mut self) -> crossterm::Result<bool> {
    let res = self.key()?;
    self.draw()?;
    Ok(res)
  }
}

