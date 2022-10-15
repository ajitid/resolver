mod buffer;
mod editor;
mod writer;
mod frame;
mod rdl;
mod text;

use std::time;
use std::io::stdout;
use std::fs;

use crossterm;
use crossterm::event;
use crossterm::execute;
use crossterm::terminal;

use clap::Parser;

use editor::Editor;
use writer::Writer;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
  #[clap(long, help="Enable debugging mode")]
  debug: bool,
  #[clap(long, help="Enable alternate screen debugging mode (no switch on exit)")]
  debug_alternate: bool,
  #[clap(long)]
  verbose: bool,
  #[clap(help="Document to open")]
  doc: Option<String>,
}

struct Finalize {
  opts: Options,
}

impl Drop for Finalize {
  fn drop(&mut self) {
    terminal::disable_raw_mode().expect("Could not finalize terminal (good luck)");
    if !self.opts.debug_alternate {
      execute!(stdout(), terminal::LeaveAlternateScreen).expect("Could not exit alternate screen");
    }
    if !self.opts.debug {
      Writer::clear().expect("Could not clear screen");
    }
  }
}

struct Reader;

impl Reader {
  fn read_key(&self) -> crossterm::Result<event::KeyEvent> {
    loop {
      if event::poll(time::Duration::from_millis(500))? {
        if let event::Event::Key(event) = event::read()? {
          return Ok(event);
        }
      }
    }
  }
}

fn main() -> crossterm::Result<()> {
  let opts = Options::parse();
  let _cleanup = Finalize{opts: opts.clone()};
  execute!(stdout(), terminal::EnterAlternateScreen)?;
  terminal::enable_raw_mode()?;
  
  let size = terminal::size().unwrap();
  let mut editor = Editor::new_with_size((size.0 as usize, size.1 as usize));
  if let Some(doc) = opts.doc {
    match fs::read_to_string(doc) {
      Ok(text) => editor.set_text(text),
      Err(err) => return Err(err.into()),
    };
  }
  
  editor.draw()?;
  loop {
    if !editor.step()? {
      break;
    }
  }
  
  Ok(())
}
