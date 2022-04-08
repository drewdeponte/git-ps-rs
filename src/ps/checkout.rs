use std::result::Result;

#[derive(Debug)]
pub enum CheckoutError {
}

pub fn checkout(repo: &git2::Repository, patch_index_optional: Option<usize>) -> Result<(), CheckoutError> {
  Ok(())
}
