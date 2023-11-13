use git2;
use std::result::Result;

#[derive(Debug)]
pub enum CommitDiffError {
    MergeCommit,
    CommitParentCountZero,
    GetParentZeroFailed,
    GetParentZeroCommitFailed,
    GetParentZeroTreeFailed,
    GetCommitTreeFailed,
    GetDiffTreeToTreeFailed,
}

impl std::fmt::Display for CommitDiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MergeCommit => write!(f, "can't diff a merge commit"),
            Self::CommitParentCountZero => write!(f, "parent count is zero"),
            Self::GetParentZeroFailed => write!(f, "failed to get parent 0 oid"),
            Self::GetParentZeroCommitFailed => write!(f, "failed to get parent 0 commit"),
            Self::GetParentZeroTreeFailed => write!(f, "failed to get parent 0 tree"),
            Self::GetCommitTreeFailed => write!(f, "failed to get tree of given commit"),
            Self::GetDiffTreeToTreeFailed => write!(f, "failed to generate diff from trees"),
        }
    }
}

impl std::error::Error for CommitDiffError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub fn commit_diff<'a>(
    repo: &'a git2::Repository,
    commit: &git2::Commit,
) -> Result<git2::Diff<'a>, CommitDiffError> {
    if commit.parent_count() > 1 {
        return Err(CommitDiffError::MergeCommit);
    }

    if commit.parent_count() > 0 {
        let parent_oid = commit
            .parent_id(0)
            .map_err(|_| CommitDiffError::GetParentZeroFailed)?;
        let parent_commit = repo
            .find_commit(parent_oid)
            .map_err(|_| CommitDiffError::GetParentZeroCommitFailed)?;
        let parent_tree = parent_commit
            .tree()
            .map_err(|_| CommitDiffError::GetParentZeroTreeFailed)?;

        let commit_tree = commit
            .tree()
            .map_err(|_| CommitDiffError::GetCommitTreeFailed)?;
        Ok(repo
            .diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), Option::None)
            .map_err(|_| CommitDiffError::GetDiffTreeToTreeFailed)?)
    } else {
        Err(CommitDiffError::CommitParentCountZero)
    }
}
