use std::str;

use crate::buffer::Buffer;

const ZERO_POS: Pos = Pos{x: 0, y: 0, index: 0};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Pos {
  index: usize,
  pub x: usize,
  pub y: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Line {
  num: usize,
  offset: usize,
  extent: usize,
  chars: usize,
  bytes: usize,
}

impl Line {
  pub fn text<'a>(&self, text: &'a str) -> &'a str {
    &text[self.offset..self.offset + self.bytes]
  }
}

pub struct Content {
  text: String,
  width: usize,
  lines: Vec<Line>,
  loc: usize,
}

impl Content {
  pub fn new(width: usize) -> Content {
    Content{
      text: String::new(),
      width: width,
      lines: Vec::new(),
      loc: 0,
    }
  }
  
  pub fn new_with_text(width: usize, text: &str) -> Content {
    let mut c = Content{
      text: text.to_owned(),
      width: width,
      lines: Vec::new(),
      loc: 0,
    };
    c.reflow();
    c
  }
  
  pub fn len(&self) -> usize {
    self.text.len()
  }
  
  pub fn lines(&self) -> str::Lines {
    self.text.lines()
  }
  
  pub fn fill(&self, b: &mut Buffer) {
    for l in &self.lines {
      let s = &self.text[l.offset..l.extent];
      b.push_str(if s.ends_with("\n") {
        &s[..s.len()-1]
      }else{
        s
      });
      b.push_str("\r\n");
    }
  }
  
  fn reflow(&mut self) -> &mut Self {
    let mut f: usize = 0;
    let mut i: usize = 0;
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut l: Vec<Line> = Vec::new();
    for c in self.text.chars() {
      i += 1;
      x += if c != '\n' { 1 } else { 0 };
      if c == '\n' || x >= self.width {
        l.push(Line{num: y, offset: f, extent: f + i, chars: x, bytes: x});
        f = f + i;
        x = 0;
        i = 0;
        y = y + 1;
      }
    }
    if i > 0 {
      l.push(Line{num: y, offset: f, extent: f + i, chars: x, bytes: x});
    }
    self.lines = l;
    self
  }
  
  pub fn up(&mut self, idx: usize) -> Pos {
    let pos = self.index(idx);
    if pos.y == 0 {
      return ZERO_POS;
    }
    let n = pos.y - 1;
    let line = &self.lines[n];
    if line.chars > pos.x {
      Pos{x: pos.x, y: n, index: line.offset + pos.x}
    }else{
      Pos{x: line.chars, y: n, index: line.extent}
    }
  }
  
  pub fn up_rel(&mut self) -> Pos {
    let pos = self.up(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn down(&mut self, idx: usize) -> Pos {
    let pos = self.index(idx);
    let n = pos.y + 1;
    if n >= self.lines.len() {
      let line = &self.lines[pos.y];
      return Pos{x: line.chars, y: pos.y, index: line.extent};
    }
    let line = &self.lines[n];
    if line.chars > pos.x {
      Pos{x: pos.x, y: n, index: line.offset + pos.x}
    }else{
      Pos{x: line.chars, y: n, index: line.extent}
    }
  }
  
  pub fn down_rel(&mut self) -> Pos {
    let pos = self.down(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn left(&mut self, idx: usize) -> Pos {
    if idx > 0 {
      self.index(idx - 1)
    }else{
      ZERO_POS
    }
  }
  
  pub fn left_rel(&mut self) -> Pos {
    let pos = self.left(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn right(&mut self, idx: usize) -> Pos {
    self.index(idx + 1)
  }
  
  pub fn right_rel(&mut self) -> Pos {
    let pos = self.right(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn home(&mut self, idx: usize) -> Pos {
    let pos = self.index(idx);
    let line = &self.lines[pos.y];
    Pos{x: 0, y: pos.y, index: line.offset}
  }
  
  pub fn home_rel(&mut self) -> Pos {
    let pos = self.home(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn end(&mut self, idx: usize) -> Pos {
    let pos = self.index(idx);
    let line = &self.lines[pos.y];
    Pos{x: line.chars, y: pos.y, index: line.extent}
  }
  
  pub fn end_rel(&mut self) -> Pos {
    let pos = self.end(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn index(&mut self, idx: usize) -> Pos {
    if idx == 0 {
      return ZERO_POS;
    }
    let l = self.len();
    let idx = if idx > l { l } else { idx };
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut nl: bool = false;
    for l in &self.lines {
      y = l.num;
      if idx >= l.offset && idx <= l.extent {
        let slice = &self.text[l.offset..l.extent];
        let eix = idx - l.offset;
        nl = true;
        for (i, c) in slice.chars().enumerate() {
          if i == eix {
            return Pos{x: i, y: l.num, index: idx};
          }
          x = i;
          nl = c == '\n';
        }
      }
    }
    if nl || x + 1 > self.width {
      Pos{x: 0, y: y+1, index: idx}
    }else{
      Pos{x: x+1, y: y, index: idx}
    }
  }
  
  pub fn insert(&mut self, idx: usize, c: char) -> Pos {
    self.text.insert(idx, c);
    return self.reflow().index(idx + 1);
  }
  
  pub fn insert_rel(&mut self, c: char) -> Pos {
    let pos = self.insert(self.loc, c);
    self.loc += 1;
    pos
  }
  
  pub fn backspace(&mut self, idx: usize) -> Pos {
    let l = self.text.len();
    if l == 0 { // nothing to delete
      return ZERO_POS;
    }
    self.text.remove(idx);
    return self.reflow().index(idx);
  }
  
  pub fn backspace_rel(&mut self) -> Pos {
    if self.loc > 0 {
      self.loc -= 1;
    }
    self.backspace(self.loc)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn test_reflow() {
    let c = Content::new_with_text(100, "Hello");
    assert_eq!(vec![
      Line{num: 0, offset: 0, extent: 5, chars: 5, bytes: 5}
    ], c.lines);
    assert_eq!(vec![
      "Hello"
    ], c.lines.iter().map(|e| { e.text(&c.text) }).collect::<Vec<&str>>());
    
    let c = Content::new_with_text(100, "Hello\nthere.");
    assert_eq!(vec![
      Line{num: 0, offset: 0, extent: 6,  chars: 5, bytes: 5},
      Line{num: 1, offset: 6, extent: 12, chars: 6, bytes: 6}
    ], c.lines);
    assert_eq!(vec![
      "Hello",
      "there.",
    ], c.lines.iter().map(|e| { e.text(&c.text) }).collect::<Vec<&str>>());

    let c = Content::new_with_text(100, "Hello\nthere.\n");
    assert_eq!(vec![
      Line{num: 0, offset: 0, extent: 6,  chars: 5, bytes: 5},
      Line{num: 1, offset: 6, extent: 13, chars: 6, bytes: 6},
    ], c.lines);
    assert_eq!(vec![
      "Hello",
      "there.",
    ], c.lines.iter().map(|e| { e.text(&c.text) }).collect::<Vec<&str>>());

    let c = Content::new_with_text(100, "Hello\nthere.\n!");
    assert_eq!(vec![
      Line{num: 0, offset: 0,  extent: 6,  chars: 5, bytes: 5},
      Line{num: 1, offset: 6,  extent: 13, chars: 6, bytes: 6},
      Line{num: 2, offset: 13, extent: 14, chars: 1, bytes: 1},
    ], c.lines);
    assert_eq!(vec![
      "Hello",
      "there.",
      "!"
    ], c.lines.iter().map(|e| { e.text(&c.text) }).collect::<Vec<&str>>());
  }
  
  #[test]
  fn test_index() {
    assert_eq!(Pos{index: 0, x: 0, y: 0}, Content::new_with_text(100, "").index(0));
    assert_eq!(Pos{index: 1, x: 1, y: 0}, Content::new_with_text(100, "H").index(1));
    assert_eq!(Pos{index: 2, x: 2, y: 0}, Content::new_with_text(100, "Hi").index(2));
    assert_eq!(Pos{index: 3, x: 0, y: 1}, Content::new_with_text(100, "Hi\n").index(3));
    assert_eq!(Pos{index: 4, x: 1, y: 1}, Content::new_with_text(100, "Hi\nT").index(4));
    assert_eq!(Pos{index: 5, x: 2, y: 1}, Content::new_with_text(100, "Hi\nTi").index(5));
    assert_eq!(Pos{index: 6, x: 3, y: 1}, Content::new_with_text(100, "Hi\nTim").index(6));
    assert_eq!(Pos{index: 7, x: 0, y: 2}, Content::new_with_text(100, "Hi\nTim\n").index(7));
    assert_eq!(Pos{index: 8, x: 1, y: 2}, Content::new_with_text(100, "Hi\nTim\n!").index(8));
    //
    assert_eq!(Pos{index: 4, x: 4, y: 0}, Content::new_with_text(100, "Hello").index(4));
    assert_eq!(Pos{index: 6, x: 6, y: 0}, Content::new_with_text(100, "Hello!\n").index(6));
    assert_eq!(Pos{index: 7, x: 0, y: 1}, Content::new_with_text(100, "Hello!\n").index(7));
  }
  
  #[test]
  fn test_movement() {
    assert_eq!(Pos{index: 6, x: 6, y: 0}, Content::new_with_text(100, "Hello.").right(5));
    assert_eq!(Pos{index: 6, x: 6, y: 0}, Content::new_with_text(100, "Hello.").right(100));
    assert_eq!(Pos{index: 7, x: 0, y: 1}, Content::new_with_text(100, "Hello,\nthere").right(6));
    
    assert_eq!(Pos{index: 4, x: 4, y: 0}, Content::new_with_text(100, "Hello.").left(5));
    assert_eq!(Pos{index: 0, x: 0, y: 0}, Content::new_with_text(100, "Hello.").left(0));
    assert_eq!(Pos{index: 6, x: 6, y: 0}, Content::new_with_text(100, "Hello,\nthere").left(7));
    
    assert_eq!(Pos{index: 0, x: 0, y: 0}, Content::new_with_text(100, "Hello,\nto\nyourself").up(7));
    assert_eq!(Pos{index: 1, x: 1, y: 0}, Content::new_with_text(100, "Hello,\nto\nyourself").up(8));
    assert_eq!(Pos{index: 10, x: 2, y: 1}, Content::new_with_text(100, "Hello,\nto\nyourself").up(13));
    assert_eq!(Pos{index: 10, x: 2, y: 1}, Content::new_with_text(100, "Hello,\nto\nyourself").up(16));

    assert_eq!(Pos{index: 10, x: 2, y: 1}, Content::new_with_text(100, "Hello,\nto\nyourself").down(2));
    assert_eq!(Pos{index: 10, x: 2, y: 1}, Content::new_with_text(100, "Hello,\nto\nyourself").down(6));
  }
  
}
