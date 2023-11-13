use git2;
use std::result::Result;

#[cfg(feature = "backup_cmd")]
#[derive(Debug)]
pub enum ReadHashedObjectError {
    NotValidUtf8(std::str::Utf8Error),
    Failed(git2::Error),
}

#[cfg(feature = "backup_cmd")]
pub fn read_hashed_object(
    repo: &git2::Repository,
    oid: git2::Oid,
) -> Result<String, ReadHashedObjectError> {
    let blob = repo.find_blob(oid).map_err(ReadHashedObjectError::Failed)?;
    let content = blob.content();
    let str_ref = std::str::from_utf8(content).map_err(ReadHashedObjectError::NotValidUtf8)?;
    Ok(str_ref.to_string())
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "backup_cmd")]
    #[test]
    fn test_read_hashed_object() {
        let (_td, repo) = repo_init();
        let message = "Hello hash object write!";
        let oid = super::hash_object_write(&repo, message).unwrap();
        let retreived_message = super::read_hashed_object(&repo, oid).unwrap();
        assert_eq!(retreived_message, message);
    }
}
