#[derive(Debug)]
pub enum CommitIsBehindError {
    GetCommitsParentZeroIdFailed(git2::Error),
}

pub fn commit_is_behind(
    commit: &git2::Commit,
    base_oid: git2::Oid,
) -> Result<bool, CommitIsBehindError> {
    if commit
        .parent_id(0)
        .map_err(CommitIsBehindError::GetCommitsParentZeroIdFailed)?
        != base_oid
    {
        Ok(true)
    } else {
        Ok(false)
    }
}
