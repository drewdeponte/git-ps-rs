use super::{line_to_rebase_todo, rebase_todo::RebaseTodoCommand};
use std::result::Result;

#[derive(Debug)]
pub enum StrToRebaseTodoError {
    Unknown(Box<dyn std::error::Error>),
}

impl std::fmt::Display for StrToRebaseTodoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(e) => write!(f, "{}", &e.to_string()),
        }
    }
}

impl std::error::Error for StrToRebaseTodoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unknown(e) => Some(e.as_ref()),
        }
    }
}

/// Parse content from a rebase todo file into a Vec<RebaseTodo> structs
///
/// Attempt to parse the given content into a Vec<RebaseTodo> structs.
pub fn str_to_rebase_todo(content: &str) -> Result<Vec<RebaseTodoCommand>, StrToRebaseTodoError> {
    content
        .lines()
        .map(line_to_rebase_todo)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| StrToRebaseTodoError::Unknown(e.into()))
}

#[cfg(test)]
mod tests {
    use super::str_to_rebase_todo;
    use super::RebaseTodoCommand;

    #[test]
    fn parses_valid_content_into_rebase_todo_lines() {
        let rebase_todos = str_to_rebase_todo(
            r#"pick 7a645dbc60057ef469b9a2bc4a82b30fa5efbb75 Free world
pick a403343e8a173dc8cc2fa27d9465b13fe2c0d627 Free new world
pick 54fd6c2a52eaf0d67eaea5adc289c0831f5f4f7e Add comment to the end
edit 939efeb6a58842e931ed0277686c9d899b2671ae Add comment to the end # empty
"#,
        )
        .unwrap();
        let expected_rebase_todos = [
            RebaseTodoCommand::Pick {
                line: "pick 7a645dbc60057ef469b9a2bc4a82b30fa5efbb75 Free world".to_string(),
                key: "pick".to_string(),
                sha: "7a645dbc60057ef469b9a2bc4a82b30fa5efbb75".to_string(),
                rest: "Free world".to_string(),
            },
            RebaseTodoCommand::Pick {
                line: "pick a403343e8a173dc8cc2fa27d9465b13fe2c0d627 Free new world".to_string(),
                key: "pick".to_string(),
                sha: "a403343e8a173dc8cc2fa27d9465b13fe2c0d627".to_string(),
                rest: "Free new world".to_string(),
            },
            RebaseTodoCommand::Pick {
                line: "pick 54fd6c2a52eaf0d67eaea5adc289c0831f5f4f7e Add comment to the end"
                    .to_string(),
                key: "pick".to_string(),
                sha: "54fd6c2a52eaf0d67eaea5adc289c0831f5f4f7e".to_string(),
                rest: "Add comment to the end".to_string(),
            },
            RebaseTodoCommand::Edit {
                line:
                    "edit 939efeb6a58842e931ed0277686c9d899b2671ae Add comment to the end # empty"
                        .to_string(),
                key: "edit".to_string(),
                sha: "939efeb6a58842e931ed0277686c9d899b2671ae".to_string(),
                rest: "Add comment to the end # empty".to_string(),
            },
        ];

        assert_eq!(rebase_todos[0], expected_rebase_todos[0]);
        assert_eq!(rebase_todos[1], expected_rebase_todos[1]);
        assert_eq!(rebase_todos[2], expected_rebase_todos[2]);
        assert_eq!(rebase_todos[3], expected_rebase_todos[3]);
    }
}
