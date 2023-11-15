#[derive(Debug, PartialEq, Eq)]
pub enum RebaseTodoCommand {
    // commands that handle single commit with no options
    Pick {
        line: String,
        key: String,
        sha: String,
        rest: String,
    },
    Revert {
        line: String,
        key: String,
        sha: String,
        rest: String,
    },
    Edit {
        line: String,
        key: String,
        sha: String,
        rest: String,
    },
    Reword {
        line: String,
        key: String,
        sha: String,
        rest: String,
    },
    Squash {
        line: String,
        key: String,
        sha: String,
        rest: String,
    },
    Drop {
        line: String,
        key: String,
        sha: String,
        rest: String,
    },
    // commands that handle single commit but have options
    Fixup {
        line: String,
        key: String,
        sha: String,
        rest: String,
        keep_only_this_commits_message: bool,
        open_editor: bool,
    },
    Merge {
        line: String,
        key: String,
        sha: Option<String>,
        label: String,
        oneline: String,
        reword: bool,
    },
    // commands that do something else than handling a single commit
    Exec {
        line: String,
        key: String,
        rest: String,
    },
    Break {
        line: String,
        key: String,
        rest: String,
    },
    Label {
        line: String,
        key: String,
        rest: String,
    },
    Reset {
        line: String,
        key: String,
        rest: String,
    },
    UpdateRef {
        line: String,
        key: String,
        rest: String,
    },
    // commands that do nothing but are counted for reporting progress
    Noop {
        line: String,
        key: String,
        rest: String,
    },
    // comments (not counted for reporting progress)
    Comment {
        line: String,
        key: String,
        message: String,
    },
}

impl std::fmt::Display for RebaseTodoCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pick {
                line,
                key: _,
                sha: _,
                rest: _,
            }
            | Self::Revert {
                line,
                key: _,
                sha: _,
                rest: _,
            }
            | Self::Edit {
                line,
                key: _,
                sha: _,
                rest: _,
            }
            | Self::Reword {
                line,
                key: _,
                sha: _,
                rest: _,
            }
            | Self::Squash {
                line,
                key: _,
                sha: _,
                rest: _,
            }
            | Self::Drop {
                line,
                key: _,
                sha: _,
                rest: _,
            }
            | Self::Fixup {
                line,
                key: _,
                sha: _,
                rest: _,
                keep_only_this_commits_message: _,
                open_editor: _,
            }
            | Self::Merge {
                line,
                key: _,
                sha: _,
                label: _,
                oneline: _,
                reword: _,
            }
            | Self::Exec {
                line,
                key: _,
                rest: _,
            }
            | Self::Break {
                line,
                key: _,
                rest: _,
            }
            | Self::Label {
                line,
                key: _,
                rest: _,
            }
            | Self::Reset {
                line,
                key: _,
                rest: _,
            }
            | Self::UpdateRef {
                line,
                key: _,
                rest: _,
            }
            | Self::Noop {
                line,
                key: _,
                rest: _,
            }
            | Self::Comment {
                line,
                key: _,
                message: _,
            } => write!(f, "{}", line),
        }
    }
}
