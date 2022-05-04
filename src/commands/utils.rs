use ansi_term::Colour::Red;

pub fn print_err(color: bool, message: &str) {
  if color {
    eprintln!("{}", Red.paint(message))
  } else {
    eprintln!("{}", message)
  }
}

