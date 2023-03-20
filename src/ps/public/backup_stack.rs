use super::super::private::git;
use std::result::Result;

#[derive(Debug)]
pub enum BackupStackError {
    OpenRepositoryFailed(git::CreateCwdRepositoryError),
    CurrentBranchNameMissing,
    GetUpstreamBranchNameFailed,
    GetRemoteNameFailed,
    ConvertStringToStrFailed,
    PushFailed(git::ExtForcePushError),
}

pub fn backup_stack(branch_name: String) -> Result<(), BackupStackError> {
    let repo = git::create_cwd_repo().map_err(BackupStackError::OpenRepositoryFailed)?;

    let cur_branch_name =
        git::get_current_branch(&repo).ok_or(BackupStackError::CurrentBranchNameMissing)?;
    let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str())
        .map_err(|_| BackupStackError::GetUpstreamBranchNameFailed)?;
    let remote_name = repo
        .branch_remote_name(&branch_upstream_name)
        .map_err(|_| BackupStackError::GetRemoteNameFailed)?;
    let remote_name_str = remote_name
        .as_str()
        .ok_or(BackupStackError::ConvertStringToStrFailed)?;

    // e.g. git push <remote> <stack-branch>:<branch-name>
    git::ext_push(true, remote_name_str, &cur_branch_name, &branch_name)
        .map_err(BackupStackError::PushFailed)?;

    Ok(())
}
