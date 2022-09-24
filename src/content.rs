use std::str;

use crossterm::event;

#[derive(Debug, Eq, PartialEq)]
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

pub struct Content {
  text: String,
  width: usize,
  input: usize,
  lines: Vec<Line>,
}

impl Content {
  pub fn new(width: usize) -> Content {
    Content{
      text: String::new(),
      width: width,
      input: 0,
      lines: Vec::new(),
    }
  }
  
  pub fn new_with_text(width: usize, text: &str) -> Content {
    let mut c = Content{
      text: text.to_owned(),
      width: width,
      input: 0,
      lines: Vec::new(),
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
  
  pub fn index(&mut self, idx: usize) -> Pos {
    if idx == 0 {
      return Pos{x: 0, y: 0, index: 0};
    }
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
    let idx = if idx > l { l } else { idx };
    self.text.insert(idx, c);
    return self.reflow().index(idx + 1);
  }
  
  pub fn insert_rel(&mut self, c: char) -> Pos {
    let pos = self.insert(self.input, c);
    self.input += 1;
    pos
  }
  
  pub fn insert_ctl(&mut self, k: event::KeyCode) -> Pos {
    match k {
      event::KeyCode::Enter => self.insert_rel('\n'),
      _ => unimplemented!(),
    }
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
    
    assert_eq!(Pos{index: 4, x: 4, y: 0}, Content::new_with_text(100, "Hello").index(4));
    assert_eq!(Pos{index: 6, x: 6, y: 0}, Content::new_with_text(100, "Hello!\n").index(6));
    assert_eq!(Pos{index: 7, x: 0, y: 1}, Content::new_with_text(100, "Hello!\n").index(7));
  }
  
}
