pub mod writer;

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
}

pub struct Editor {
  reader: Reader,
  writer: Writer,
  text: Text,
  mode: Mode,
  pos: Pos,
}

impl Editor {
  pub fn new_with_size(size: (usize, usize), opts: options::Options) -> Self {
    Editor{
      reader: Reader,
      writer: Writer::new_with_size(size, opts),
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
      } => self.pos = self.text.edit_rel(Action{
        movement: Movement::Left,
        operation: op,
      }),
      event::KeyEvent{
        code: event::KeyCode::Right,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.edit_rel(Action{
        movement: Movement::Right,
        operation: op,
      }),
      event::KeyEvent{
        code: event::KeyCode::Up,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.edit_rel(Action{
        movement: Movement::Up,
        operation: op,
      }),
      event::KeyEvent{
        code: event::KeyCode::Down,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.edit_rel(Action{
        movement: Movement::Down,
        operation: op,
      }),
      event::KeyEvent{
        code: event::KeyCode::Home,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.edit_rel(Action{
        movement: Movement::StartOfLine,
        operation: op,
      }),
      event::KeyEvent{
        code: event::KeyCode::End,
        modifiers: event::KeyModifiers::NONE,
        ..
      } => self.pos = self.text.edit_rel(Action{
        movement: Movement::EndOfLine,
        operation: op,
      }),

      event::KeyEvent{
        code: event::KeyCode::Char('b'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => self.pos = self.text.edit_rel(Action{
        movement: Movement::StartOfWord,
        operation: op,
      }),
      event::KeyEvent{
        code: event::KeyCode::Char('e'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => self.pos = self.text.edit_rel(Action{
        movement: Movement::EndOfWord,
        operation: op,
      }),
      event::KeyEvent{
        code: event::KeyCode::Char('w'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => self.pos = self.text.edit_rel(Action{
        movement: Movement::Word,
        operation: op,
      }),
      
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
    
    // mode resets after operation in all cases
    self.mode = Mode::Normal;
    
    Ok(true)
  }

/*
  pub fn key(&mut self) -> crossterm::Result<bool> {
    match self.reader.read_key()? {
      event::KeyEvent{
        code: event::KeyCode::Char('q'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => return Ok(false),
      event::KeyEvent{
        code: event::KeyCode::Char('d'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => self.mode = Mode::Delete,
      
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
        code: event::KeyCode::Char('b'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => self.pos = self.text.to_rel(action::Movement::StartOfWord),
      event::KeyEvent{
        code: event::KeyCode::Char('e'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => self.pos = self.text.to_rel(action::Movement::EndOfWord),
      event::KeyEvent{
        code: event::KeyCode::Char('w'),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => self.pos = self.text.to_rel(action::Movement::Word),
      
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
*/  
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

