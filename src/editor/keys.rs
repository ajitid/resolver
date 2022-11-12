use crossterm::event;

pub enum Movement {
  Up,
  Right,
  Down,
  Left,
  StartOfLine,
  EndOfLine,
}

pub enum Operation {
  Move,
  Select,
  Delete,
}

pub struct Event {
  movement: Movement,
  operation: Operation,
}
