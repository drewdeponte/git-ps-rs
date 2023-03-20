mod execute;
mod mergable;
mod print_warn;
mod string_manipulation;

pub use execute::{execute, execute_with_output, ExecuteError, ExecuteWithOutputError};
pub use mergable::merge_option;
pub use mergable::Mergable;
pub use print_warn::print_warn;
pub use string_manipulation::{set_string_width, strip_newlines};
