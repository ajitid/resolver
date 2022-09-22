use std::io;
use std::io::Read;

use crossterm::terminal;

struct Finalize;

impl Drop for Finalize {
  fn drop(&mut self) {
    terminal::disable_raw_mode().expect("Could not finalize terminal (good luck)");
  }
}

fn main() {
  let _cleanup = Finalize{};
  terminal::enable_raw_mode().expect("Could not initialize terminal");
  
  let mut buf = [0; 1];
  while io::stdin().read(&mut buf).expect("Failed to read line") == 1 {
    if buf[0] == b'q' {
      break;
    }
    let c = buf[0] as char;
    if c.is_control() {
      println!("{}\r", c as u8);
    }else{
      println!("{}\r", c);
    }
  }
}
