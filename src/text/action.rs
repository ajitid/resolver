
pub enum Movement {
  Up,
  Right,
  Down,
  Left,
  Word,
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
  pub movement: Movement,
  pub operation: Operation,
}

impl Action {
  pub fn new(mv: Movement, op: Operation) -> Action {
    Action{
      movement: mv,
      operation: op,
    }
  }
}