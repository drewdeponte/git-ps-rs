use std::fmt;
use ansi_term::ANSIGenericString;

use super::utils;

struct ListCell {
  width: Option<usize>,
  color: Option<ansi_term::Colour>,
  value: String,
}

impl ListCell {
  fn get_str_fixed_width(&self, str: String) -> String {
    return self.width.map(|w| utils::set_string_width(&str, w)).unwrap_or(str);
  }

  fn get_colored_text<'a>(&'a self, str: &'a str) -> ANSIGenericString<str> {
    return self.color.map(|c| c.paint(str)).unwrap_or(ANSIGenericString::from(str));
  }
}

impl fmt::Display for ListCell {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let without_newlines = utils::strip_newlines(&self.value);
    let str_fixed_width = self.get_str_fixed_width(without_newlines);
    let colored_text = self.get_colored_text(&str_fixed_width);
    return write!(f, "{}", colored_text);
  }
}

#[cfg(test)]
mod tests {
  use crate::ps::private::list::ListCell;
  use ansi_term::Colour::Blue;

  #[test]
  fn test_list_cell_fmt_shorter_blue() {
    let cell = ListCell { width: Some(4), color: Some(Blue), value: "hello".to_string() };
    assert_eq!(format!("{}", cell), "\u{1b}[34mhell\u{1b}[0m");
  }

  #[test]
  fn test_list_cell_fmt_longer_no_color() {
    let cell = ListCell { width: Some(6), color: None, value: "hello".to_string() };
    assert_eq!(format!("{}", cell), "hello ");
  }

  #[test]
  fn test_list_cell_fmt_no_width_or_color() {
    let cell = ListCell { width: None, color: None, value: "hello".to_string() };
    assert_eq!(format!("{}", cell), "hello");
  }
}
