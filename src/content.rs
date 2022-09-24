use std::str;

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
  index: usize,
  length: usize,
}

impl Line {
  pub fn upper(&self) -> usize {
    self.index + self.length
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
  
  pub fn _cursor(&self) -> usize {
    self.loc
  }
  
  fn reflow(&mut self) -> &mut Self {
    let mut f: usize = 0;
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut l: Vec<Line> = Vec::new();
    for c in self.text.chars() {
      x = x + 1;
      if c == '\n' || x >= self.width {
        l.push(Line{
          num: y,
          index: f,
          length: x,
        });
        f = f + x;
        x = 0;
        y = y + 1;
      }
    }
    if x > 0 {
      l.push(Line{
        num: y,
        index: f,
        length: x,
      });
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
    if line.length > pos.x {
      Pos{x: pos.x, y: n, index: line.index + pos.x}
    }else{
      Pos{x: line.length - 1, y: n, index: line.upper() - 1}
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
      return Pos{x: line.length, y: pos.y, index: line.upper() - 1};
    }
    let line = &self.lines[n];
    if line.length > pos.x {
      Pos{x: pos.x, y: n, index: line.index + pos.x}
    }else{
      Pos{x: line.length - 1, y: n, index: line.upper() - 1}
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
      let ub = l.index + l.length;
      y = l.num;
      if idx >= l.index && idx <= ub {
        let slice = &self.text[l.index..ub];
        let eix = idx - l.index;
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
    let l = self.text.len();
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
    assert_eq!(vec![
      Line{num: 0, index: 0, length: 5}
    ], Content::new_with_text(100, "Hello").lines);
    assert_eq!(vec![
      Line{num: 0, index: 0, length: 6},
      Line{num: 1, index: 6, length: 6}
    ], Content::new_with_text(100, "Hello\nthere.").lines);
    assert_eq!(vec![
      Line{num: 0, index: 0, length: 6},
      Line{num: 1, index: 6, length: 7},
    ], Content::new_with_text(100, "Hello\nthere.\n").lines);
    assert_eq!(vec![
      Line{num: 0, index: 0,  length: 6},
      Line{num: 1, index: 6,  length: 7},
      Line{num: 2, index: 13, length: 1},
    ], Content::new_with_text(100, "Hello\nthere.\n!").lines);
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
    assert_eq!(Pos{index: 9, x: 2, y: 1}, Content::new_with_text(100, "Hello,\nto\nyourself").up(13));
    assert_eq!(Pos{index: 9, x: 2, y: 1}, Content::new_with_text(100, "Hello,\nto\nyourself").up(16));

    assert_eq!(Pos{index: 9, x: 2, y: 1}, Content::new_with_text(100, "Hello,\nto\nyourself").down(2));
    assert_eq!(Pos{index: 9, x: 2, y: 1}, Content::new_with_text(100, "Hello,\nto\nyourself").down(6));
  }
  
}
