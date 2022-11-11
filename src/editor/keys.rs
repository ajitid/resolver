use crossterm::event;

enum Movement {
  Up,
  Right,
  Down,
  Left,
  StartOfLine,
  EndOfLine,
}

enum Operation {
  Move,
  Select,
  Delete,
}

//struct 
