use ansi_term::Colour::Red;

pub fn print_err(color: bool, message: &str) {
    if color {
        eprintln!("{}", Red.paint(message))
    } else {
        eprintln!("{}", message)
    }
}

pub fn print_error_chain(color: bool, e: Box<dyn std::error::Error>) {
    print_err(color, &format!("\nError: {}\n", e));
    let mut err = Some(e.as_ref());
    while let Some(e) = err {
        print_err(color, &format!("Caused by: {}\n", e));
        err = e.source();
    }
}
