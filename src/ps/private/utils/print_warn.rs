use ansi_term::Colour::Yellow;

pub fn print_warn(color: bool, message: &str) {
    if color {
        eprintln!("{}", Yellow.paint(message))
    } else {
        eprintln!("{}", message)
    }
}
