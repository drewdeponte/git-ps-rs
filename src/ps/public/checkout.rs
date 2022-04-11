use super::super::private::git;
use super::super::private;

#[derive(Debug)]
pub enum CheckoutError {
  OpenRepositoryFailed(git::CreateCwdRepositoryError),
  CheckoutOperationFailed(private::checkout::CheckoutError)
}

pub fn checkout(patch_index: usize) -> Result<(), CheckoutError> {
  let repo = git::create_cwd_repo().map_err(CheckoutError::OpenRepositoryFailed)?;
  private::checkout::checkout(&repo, patch_index).map_err(CheckoutError::CheckoutOperationFailed)?;
  Ok(())
}
