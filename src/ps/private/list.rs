use super::{utils, config, paths, hooks::{get_hook_output, HookOutputError}};

#[derive(Debug)]
pub enum GetAdditionalInfoHookOutputError {
  GetRepoRootPathFailed(paths::PathsError),
  PathNotUtf8,
  GetConfigFailed(config::GetConfigError),
  GetHookOutputError(HookOutputError),
}

pub fn get_additional_info_hook_output(repo: &git2::Repository, patch_args: &[&str]) -> Result<String, GetAdditionalInfoHookOutputError> {
  let repo_root_path = paths::repo_root_path(&repo).map_err(GetAdditionalInfoHookOutputError::GetRepoRootPathFailed)?;
  let repo_root_str = repo_root_path.to_str().ok_or(GetAdditionalInfoHookOutputError::PathNotUtf8)?;
  let config = config::get_config(repo_root_str).map_err(GetAdditionalInfoHookOutputError::GetConfigFailed)?;

  let hook_stdout_str_result = get_hook_output(repo, "list_additional_information", patch_args);

  let hook_stdout_str = hook_stdout_str_result.map_err(GetAdditionalInfoHookOutputError::GetHookOutputError)?;

  let hook_stdout_len = config.list.extra_patch_info_length;
  return Ok(utils::set_string_width(&utils::strip_newlines(&hook_stdout_str), hook_stdout_len));
}