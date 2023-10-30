use ansi_term::{ANSIGenericString, Style};
use std::{fmt, str::Utf8Error};

use super::{hooks, utils};

#[derive(Debug, PartialEq, Clone)]
struct ListCell {
    width: Option<usize>,
    color: Option<ansi_term::Colour>,
    bg_color: Option<ansi_term::Colour>,
    value: String,
}

impl ListCell {
    fn get_str_fixed_width(&self, str: String) -> String {
        self.width
            .map(|w| utils::set_string_width(&str, w))
            .unwrap_or(str)
    }

    fn get_colored_text<'a>(&'a self, str: &'a str) -> ANSIGenericString<str> {
        if self.color.is_some() && self.bg_color.is_some() {
            self.color.unwrap().on(self.bg_color.unwrap()).paint(str)
        } else if self.color.is_some() && self.bg_color.is_none() {
            self.color.unwrap().paint(str)
        } else if self.color.is_none() & self.bg_color.is_some() {
            Style::new().on(self.bg_color.unwrap()).paint(str)
        } else {
            ANSIGenericString::from(str)
        }
    }
}

impl fmt::Display for ListCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let without_newlines = utils::strip_newlines(&self.value);
        let str_fixed_width = self.get_str_fixed_width(without_newlines);
        let colored_text = self.get_colored_text(&str_fixed_width);
        write!(f, "{}", colored_text)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ListRow {
    cells: Vec<ListCell>,
    with_color: bool,
}

impl ListRow {
    pub fn new(with_color: bool) -> Self {
        Self {
            with_color,
            cells: vec![],
        }
    }

    pub fn add_cell(
        &mut self,
        width: Option<usize>,
        text_color: Option<ansi_term::Colour>,
        bg_color: Option<ansi_term::Colour>,
        value: impl fmt::Display,
    ) {
        let color = if self.with_color { text_color } else { None };
        let bg_color = if self.with_color { bg_color } else { None };
        let cell = ListCell {
            width,
            color,
            bg_color,
            value: value.to_string(),
        };
        self.cells.push(cell)
    }
}

impl fmt::Display for ListRow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut row_str = String::new();

        for column in &self.cells {
            row_str.push_str(&column.to_string());
        }
        write!(f, "{}", row_str)
    }
}

#[derive(Debug)]
pub enum ListHookError {
    GetHookOutputError(hooks::HookOutputError),
    HookOutputInvalid(Utf8Error),
}

impl std::fmt::Display for ListHookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GetHookOutputError(e) => write!(f, "get hook output failed, {}", e),
            Self::HookOutputInvalid(e) => write!(f, "hook output invalid, {}", e),
        }
    }
}

impl std::error::Error for ListHookError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GetHookOutputError(e) => Some(e),
            Self::HookOutputInvalid(e) => Some(e),
        }
    }
}

pub fn execute_list_additional_info_hook(
    repo_root_str: &str,
    repo_gitdir_str: &str,
    args: &[&str],
) -> Result<String, ListHookError> {
    let hook_output = hooks::find_and_execute_hook_with_output(
        repo_root_str,
        repo_gitdir_str,
        "list_additional_information",
        args,
    )
    .map_err(ListHookError::GetHookOutputError)?;
    String::from_utf8(hook_output.stdout)
        .map_err(|e| ListHookError::HookOutputInvalid(e.utf8_error()))
}

#[cfg(test)]
mod tests {
    use crate::ps::private::list::{ListCell, ListRow};
    use ansi_term::Colour::Blue;

    #[test]
    fn test_list_cell_fmt_shorter_blue() {
        let cell = ListCell {
            width: Some(4),
            color: Some(Blue),
            bg_color: None,
            value: "hello".to_string(),
        };
        assert_eq!(format!("{}", cell), "\u{1b}[34mhell\u{1b}[0m");
    }

    #[test]
    fn test_list_cell_fmt_longer_no_color() {
        let cell = ListCell {
            width: Some(6),
            color: None,
            bg_color: None,
            value: "hello".to_string(),
        };
        assert_eq!(format!("{}", cell), "hello ");
    }

    #[test]
    fn test_list_cell_fmt_no_width_or_color() {
        let cell = ListCell {
            width: None,
            color: None,
            bg_color: None,
            value: "hello".to_string(),
        };
        assert_eq!(format!("{}", cell), "hello");
    }

    #[test]
    fn test_list_row_new() {
        let row = ListRow::new(true);
        assert_eq!(
            row,
            ListRow {
                with_color: true,
                cells: vec![]
            }
        );
    }

    #[test]
    fn test_list_row_add_cell() {
        let mut row = ListRow::new(false);
        let cell_value = "hello".to_string();
        row.add_cell(None, None, None, &cell_value);
        assert_eq!(
            row,
            ListRow {
                with_color: false,
                cells: vec![ListCell {
                    width: None,
                    color: None,
                    bg_color: None,
                    value: cell_value
                }]
            }
        );
    }

    #[test]
    fn test_list_row_fmt_with_color() {
        let mut row = ListRow::new(true);
        let first_cell = ListCell {
            width: Some(10),
            color: Some(Blue),
            bg_color: None,
            value: "Hello".to_string(),
        };
        let second_cell = ListCell {
            width: None,
            color: None,
            bg_color: None,
            value: "World".to_string(),
        };
        row.add_cell(
            first_cell.width,
            first_cell.color,
            first_cell.bg_color,
            first_cell.value,
        );
        row.add_cell(
            second_cell.width,
            second_cell.color,
            second_cell.bg_color,
            second_cell.value,
        );
        assert_eq!(format!("{}", row), "\u{1b}[34mHello     \u{1b}[0mWorld")
    }

    #[test]
    fn test_list_row_fmt_without_color() {
        let mut row = ListRow::new(false);
        let first_cell = ListCell {
            width: Some(10),
            color: Some(Blue),
            bg_color: None,
            value: "Hello".to_string(),
        };
        let second_cell = ListCell {
            width: None,
            color: None,
            bg_color: None,
            value: "World".to_string(),
        };
        row.add_cell(
            first_cell.width,
            first_cell.color,
            first_cell.bg_color,
            first_cell.value,
        );
        row.add_cell(
            second_cell.width,
            second_cell.color,
            second_cell.bg_color,
            second_cell.value,
        );
        assert_eq!(format!("{}", row), "Hello     World")
    }
}
