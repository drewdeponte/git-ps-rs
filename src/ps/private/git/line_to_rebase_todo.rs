use super::rebase_todo::RebaseTodoCommand;
use std::result::Result;

#[derive(Debug, PartialEq, Eq)]
pub enum LineToRebaseTodoError {
    UnknownCommand { command: String, line: String },
}

impl std::fmt::Display for LineToRebaseTodoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCommand { command, line } => {
                write!(f, "unkown command: {}, in line: {}", command, line)
            }
        }
    }
}

impl std::error::Error for LineToRebaseTodoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnknownCommand {
                command: _,
                line: _,
            } => None,
        }
    }
}

fn str_to_whitespace_or_eol(s: &str) -> (&str, Option<&str>) {
    match s.find(char::is_whitespace) {
        Some(idx) => {
            let rest = &s[(idx + 1)..];
            if rest.is_empty() {
                (&s[..idx], None)
            } else {
                (&s[..idx], Some(rest))
            }
        }
        None => (s, None),
    }
}

/// Parse line from a rebase todo file into a RebaseTodo struct
///
/// Attempt to parse the given line into a RebaseTodo struct.
pub fn line_to_rebase_todo(line: &str) -> Result<RebaseTodoCommand, LineToRebaseTodoError> {
    let commit_based_commands = [
        "pick", "p", "revert", "edit", "e", "reword", "r", "squash", "s", "drop", "d",
    ];

    let trimmed_line = line.trim();

    let first_char = trimmed_line.chars().next();

    // short circuit for empty lines
    if first_char.is_none() {
        return Ok(RebaseTodoCommand::Comment {
            line: trimmed_line.to_string(),
            key: "#".to_string(),
            message: "empty line".to_string(),
        });
    }

    // short circuit for comments
    if first_char.is_some() && first_char.unwrap() == '#' {
        let rest = trimmed_line[1..].trim();
        return Ok(RebaseTodoCommand::Comment {
            line: trimmed_line.to_string(),
            key: "#".to_string(),
            message: rest.to_string(),
        });
    }

    let (cmd, rest_after_cmd) = str_to_whitespace_or_eol(trimmed_line);

    // parse commit associated commands
    if commit_based_commands.contains(&cmd) {
        let rest = rest_after_cmd.expect("sha to follow the command");

        let (sha, rest_after_sha) = str_to_whitespace_or_eol(rest);

        return match cmd {
            "pick" | "p" => Ok(RebaseTodoCommand::Pick {
                line: line.to_string(),
                key: cmd.to_string(),
                sha: sha.to_string(),
                rest: rest_after_sha.unwrap_or("").to_string(),
            }),
            "revert" => Ok(RebaseTodoCommand::Revert {
                line: line.to_string(),
                key: cmd.to_string(),
                sha: sha.to_string(),
                rest: rest_after_sha.unwrap_or("").to_string(),
            }),
            "edit" | "e" => Ok(RebaseTodoCommand::Edit {
                line: line.to_string(),
                key: cmd.to_string(),
                sha: sha.to_string(),
                rest: rest_after_sha.unwrap_or("").to_string(),
            }),
            "reword" | "r" => Ok(RebaseTodoCommand::Reword {
                line: line.to_string(),
                key: cmd.to_string(),
                sha: sha.to_string(),
                rest: rest_after_sha.unwrap_or("").to_string(),
            }),
            "squash" | "s" => Ok(RebaseTodoCommand::Squash {
                line: line.to_string(),
                key: cmd.to_string(),
                sha: sha.to_string(),
                rest: rest_after_sha.unwrap_or("").to_string(),
            }),
            "drop" | "d" => Ok(RebaseTodoCommand::Drop {
                line: line.to_string(),
                key: cmd.to_string(),
                sha: sha.to_string(),
                rest: rest_after_sha.unwrap_or("").to_string(),
            }),
            &_ => Err(LineToRebaseTodoError::UnknownCommand {
                command: cmd.to_string(),
                line: line.to_string(),
            }),
        };
    }

    // parse fixup (commit associated command with options)
    if cmd == "fixup" || cmd == "f" {
        let rest = rest_after_cmd.expect("more to follow the command");

        let (next_token, rest_after_next_token) = str_to_whitespace_or_eol(rest);

        return Ok(match next_token {
            "-C" => {
                let (sha, fixup_rest_after_sha) =
                    str_to_whitespace_or_eol(rest_after_next_token.expect("sha after -C"));
                RebaseTodoCommand::Fixup {
                    line: line.to_string(),
                    key: cmd.to_string(),
                    sha: sha.to_string(),
                    keep_only_this_commits_message: true,
                    open_editor: false,
                    rest: fixup_rest_after_sha.unwrap_or("").to_string(),
                }
            }
            "-c" => {
                let (sha, fixup_rest_after_sha) =
                    str_to_whitespace_or_eol(rest_after_next_token.expect("sha after -c"));
                RebaseTodoCommand::Fixup {
                    line: line.to_string(),
                    key: cmd.to_string(),
                    sha: sha.to_string(),
                    keep_only_this_commits_message: true,
                    open_editor: true,
                    rest: fixup_rest_after_sha.unwrap_or("").to_string(),
                }
            }
            _ => {
                let (sha, fixup_rest_after_sha) =
                    str_to_whitespace_or_eol(rest_after_next_token.expect("sha after command"));
                RebaseTodoCommand::Fixup {
                    line: line.to_string(),
                    key: cmd.to_string(),
                    sha: sha.to_string(),
                    keep_only_this_commits_message: false,
                    open_editor: false,
                    rest: fixup_rest_after_sha.unwrap_or("").to_string(),
                }
            }
        });
    }

    // parse merge (commit associated command with options)
    if cmd == "merge" || cmd == "m" {
        let rest = rest_after_cmd.expect("more to follow the command");

        let (next_token, rest_after_next_token) = str_to_whitespace_or_eol(rest);

        return Ok(match next_token {
            "-C" => {
                let (sha, merge_rest_after_sha) =
                    str_to_whitespace_or_eol(rest_after_next_token.expect("sha after -C"));

                let (label, rest_after_label) =
                    str_to_whitespace_or_eol(merge_rest_after_sha.expect("label after sha"));

                RebaseTodoCommand::Merge {
                    line: line.to_string(),
                    key: cmd.to_string(),
                    sha: Some(sha.to_string()),
                    label: label.to_string(),
                    oneline: rest_after_label.unwrap_or("").to_string(),
                    reword: false,
                }
            }
            "-c" => {
                let (sha, merge_rest_after_sha) =
                    str_to_whitespace_or_eol(rest_after_next_token.expect("sha after -C"));

                let (label, rest_after_label) =
                    str_to_whitespace_or_eol(merge_rest_after_sha.expect("label after sha"));

                RebaseTodoCommand::Merge {
                    line: line.to_string(),
                    key: cmd.to_string(),
                    sha: Some(sha.to_string()),
                    label: label.to_string(),
                    oneline: rest_after_label.unwrap_or("").to_string(),
                    reword: true,
                }
            }
            _ => {
                let (label, rest_after_label) =
                    str_to_whitespace_or_eol(rest_after_next_token.expect("label after command"));

                RebaseTodoCommand::Merge {
                    line: line.to_string(),
                    key: cmd.to_string(),
                    sha: None,
                    label: label.to_string(),
                    oneline: rest_after_label.unwrap_or("").to_string(),
                    reword: false,
                }
            }
        });
    }

    // parse non-commit associated commands
    match cmd {
        "x" | "exec" => Ok(RebaseTodoCommand::Exec {
            line: line.to_string(),
            key: cmd.to_string(),
            rest: rest_after_cmd.unwrap_or("").to_string(),
        }),
        "b" | "break" => Ok(RebaseTodoCommand::Break {
            line: line.to_string(),
            key: cmd.to_string(),
            rest: rest_after_cmd.unwrap_or("").to_string(),
        }),

        "l" | "label" => Ok(RebaseTodoCommand::Label {
            line: line.to_string(),
            key: cmd.to_string(),
            rest: rest_after_cmd.unwrap_or("").to_string(),
        }),
        "t" | "reset" => Ok(RebaseTodoCommand::Reset {
            line: line.to_string(),
            key: cmd.to_string(),
            rest: rest_after_cmd.unwrap_or("").to_string(),
        }),
        "u" | "update-ref" => Ok(RebaseTodoCommand::UpdateRef {
            line: line.to_string(),
            key: cmd.to_string(),
            rest: rest_after_cmd.unwrap_or("").to_string(),
        }),
        "n" | "noop" => Ok(RebaseTodoCommand::Noop {
            line: line.to_string(),
            key: cmd.to_string(),
            rest: rest_after_cmd.unwrap_or("").to_string(),
        }),
        &_ => Err(LineToRebaseTodoError::UnknownCommand {
            command: cmd.to_string(),
            line: line.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::line_to_rebase_todo;
    use super::RebaseTodoCommand;

    #[test]
    fn parses_valid_rebase_todo_line() {
        let rebase_todo =
            line_to_rebase_todo("pick a403343e8a173dc8cc2fa27d9465b13fe2c0d627 Free new world")
                .unwrap();
        let expected_rebase_todo = RebaseTodoCommand::Pick {
            line: "pick a403343e8a173dc8cc2fa27d9465b13fe2c0d627 Free new world".to_string(),
            key: "pick".to_string(),
            sha: "a403343e8a173dc8cc2fa27d9465b13fe2c0d627".to_string(),
            rest: "Free new world".to_string(),
        };

        assert_eq!(rebase_todo, expected_rebase_todo);
    }

    #[test]
    fn parses_valid_rebase_todo_line_with_ending_newline() {
        let rebase_todo =
            line_to_rebase_todo("pick a403343e8a173dc8cc2fa27d9465b13fe2c0d627 Free new world\n")
                .unwrap();
        let expected_rebase_todo = RebaseTodoCommand::Pick {
            line: "pick a403343e8a173dc8cc2fa27d9465b13fe2c0d627 Free new world\n".to_string(),
            key: "pick".to_string(),
            sha: "a403343e8a173dc8cc2fa27d9465b13fe2c0d627".to_string(),
            rest: "Free new world".to_string(),
        };

        assert_eq!(rebase_todo, expected_rebase_todo);
    }

    #[test]
    fn parses_valid_rebase_todo_line_missing_a_summary() {
        let rebase_todo =
            line_to_rebase_todo("pick a403343e8a173dc8cc2fa27d9465b13fe2c0d627").unwrap();
        let expected_rebase_todo = RebaseTodoCommand::Pick {
            line: "pick a403343e8a173dc8cc2fa27d9465b13fe2c0d627".to_string(),
            key: "pick".to_string(),
            sha: "a403343e8a173dc8cc2fa27d9465b13fe2c0d627".to_string(),
            rest: "".to_string(),
        };

        assert_eq!(rebase_todo, expected_rebase_todo);
    }

    #[test]
    fn parses_valid_rebase_todo_line_missing_a_summary_with_ending_newline() {
        let rebase_todo =
            line_to_rebase_todo("pick a403343e8a173dc8cc2fa27d9465b13fe2c0d627\n").unwrap();
        let expected_rebase_todo = RebaseTodoCommand::Pick {
            line: "pick a403343e8a173dc8cc2fa27d9465b13fe2c0d627\n".to_string(),
            key: "pick".to_string(),
            sha: "a403343e8a173dc8cc2fa27d9465b13fe2c0d627".to_string(),
            rest: "".to_string(),
        };

        assert_eq!(rebase_todo, expected_rebase_todo);
    }
}
