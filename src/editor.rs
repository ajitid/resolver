use crossterm::event;

use crate::Reader;
use crate::error;
use crate::text::{self, Text, Pos};
use crate::writer::Writer;
use crate::options;

pub struct Editor {
  _opts: options::Options,
  reader: Reader,
  writer: Writer,
  text: Text,
  pos: Pos,
}

impl Editor {
  pub fn new_with_size(size: (usize, usize), opts: options::Options) -> Self {
    Editor{
      _opts: opts.clone(),
      reader: Reader,
      writer: Writer::new_with_size(size, opts),
      text: Text::new((size.0 / 3) * 2),
      pos: text::ZERO_POS,
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
      } => self.pos = self.text.left_rel(),
      event::KeyEvent{
        code: event::KeyCode::Right,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.right_rel(),
      event::KeyEvent{
        code: event::KeyCode::Up,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.up_rel(),
      event::KeyEvent{
        code: event::KeyCode::Down,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.down_rel(),
      event::KeyEvent{
        code: event::KeyCode::Home,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.home_rel(),
      event::KeyEvent{
        code: event::KeyCode::End,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.end_rel(),
      event::KeyEvent{
        code: event::KeyCode::Backspace,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.backspace_rel(),
      event::KeyEvent{
        code: event::KeyCode::Char(v),
        modifiers: event::KeyModifiers::NONE | event::KeyModifiers::SHIFT,
        ..
      } => self.pos = self.text.insert_rel(v),
      event::KeyEvent{
        code: event::KeyCode::Enter,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.insert_rel('\n'),
      _ => {},
    };
    Ok(true)
  }
  
  pub fn draw(&mut self) -> Result<bool, error::Error> {
    self.writer.refresh(&self.pos, &self.text)?;
    Ok(true)
  }
  
  pub fn step(&mut self) -> Result<bool, error::Error> {
    let res = self.key()?;
    self.draw()?;
    Ok(res)
  }
}

