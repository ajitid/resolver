pub mod writer;

use std::path;

use crossterm::event;

use writer::Writer;

use crate::Reader;
use crate::error;
use crate::text::{self, Text, Pos};
use crate::text::action::{Action, Movement, Operation};
use crate::options;

enum Mode {
  Normal,
  Delete,
  // Select,
}

pub struct Editor {
  reader: Reader,
  writer: Writer,
  file: Option<path::PathBuf>,
  text: Text,
  mode: Mode,
  pos: Pos,
}

impl Editor {
  pub fn new_with_size(size: (usize, usize), opts: options::Options) -> Self {
    Editor{
      reader: Reader,
      writer: Writer::new_with_size(size, opts),
      file: None,
      text: Text::new((size.0 / 3) * 2),
      mode: Mode::Normal,
      pos: text::ZERO_POS,
    }
  }
  
  pub fn set_text(&mut self, text: String) {
    self.text.set_text(text)
  }
  
  pub fn key(&mut self) -> crossterm::Result<bool> {
    let evt = self.reader.read_key()?;
    let op = match self.mode {
      Mode::Normal => Operation::Move,
      Mode::Delete => Operation::Delete,
    };
    match evt {
      event::KeyEvent{
        code: event::KeyCode::Char('q'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => return Ok(false),
      
      event::KeyEvent{
        code: event::KeyCode::Char('d'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => {
        self.mode = Mode::Delete;
        return Ok(true);
      },
      
      event::KeyEvent{
        code: event::KeyCode::Left,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.edit_rel(Action::new(Movement::Left, op)),
      event::KeyEvent{
        code: event::KeyCode::Right,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.edit_rel(Action::new(Movement::Right, op)),
      event::KeyEvent{
        code: event::KeyCode::Up,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.edit_rel(Action::new(Movement::Up, op)),
      event::KeyEvent{
        code: event::KeyCode::Down,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.edit_rel(Action::new(Movement::Down, op)),
      event::KeyEvent{
        code: event::KeyCode::Home,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.edit_rel(Action::new(Movement::StartOfLine, op)),
      event::KeyEvent{
        code: event::KeyCode::End,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.edit_rel(Action::new(Movement::EndOfLine, op)),

      event::KeyEvent{
        code: event::KeyCode::Char('b'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => self.pos = self.text.edit_rel(Action::new(Movement::StartOfWord, op)),
      event::KeyEvent{
        code: event::KeyCode::Char('e'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => self.pos = self.text.edit_rel(Action::new(Movement::EndOfWord, op)),
      event::KeyEvent{
        code: event::KeyCode::Char('w'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => self.pos = self.text.edit_rel(Action::new(Movement::Word, op)),
      
      event::KeyEvent{
        code: event::KeyCode::Backspace,
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => self.pos = self.text.edit_rel(Action::new(Movement::StartOfWord, Operation::Delete)),
      event::KeyEvent{
        code: event::KeyCode::Backspace,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.edit_rel(Action::new(Movement::Left, Operation::Delete)),
      event::KeyEvent{
        code: event::KeyCode::Delete,
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => self.pos = self.text.edit_rel(Action::new(Movement::EndOfWord, Operation::Delete)),
      event::KeyEvent{
        code: event::KeyCode::Delete,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.edit_rel(Action::new(Movement::Right, Operation::Delete)),
      
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
      event::KeyEvent{
        code: event::KeyCode::Tab,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.insert_rel(' '),

      _ => {},
    };
    
    // mode resets after operation in all cases
    self.mode = Mode::Normal;
    
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

