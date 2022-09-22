use crossterm;
use crossterm::terminal;
use crossterm::event::{self, Event, KeyCode, KeyEvent};

struct Finalize;

impl Drop for Finalize {
  fn drop(&mut self) {
    terminal::disable_raw_mode().expect("Could not finalize terminal (good luck)");
  }
}

fn main() -> crossterm::Result<()> {
  let _cleanup = Finalize{};
  terminal::enable_raw_mode()?;
  
  loop {
    if let Event::Key(event) = event::read()? {
      match event {
        KeyEvent{
          code: KeyCode::Char('q'),
          modifiers: event::KeyModifiers::NONE,
          ..
        } => break,
        _ => {
          
        }
      };
      println!("{:?}\r", event);
    }
  }
  
  Ok(())
}
