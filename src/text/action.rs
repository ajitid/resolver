
pub enum Movement {
  Up,
  Right,
  Down,
  Left,
  StartOfWord,
  EndOfWord,
  StartOfLine,
  EndOfLine,
}

pub enum Operation {
  Move,
  Select,
  Delete,
}

pub struct Action {
  movement: Movement,
  operation: Operation,
}

