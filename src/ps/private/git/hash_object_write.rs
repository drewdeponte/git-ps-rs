use git2;
use std::result::Result;

#[cfg(feature = "backup_cmd")]
#[derive(Debug)]
pub enum HashObjectWriteError {
    Failed(git2::Error),
}

#[cfg(feature = "backup_cmd")]
pub fn hash_object_write(
    repo: &git2::Repository,
    content: &str,
) -> Result<git2::Oid, HashObjectWriteError> {
    repo.blob(content.as_bytes())
        .map_err(HashObjectWriteError::Failed)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "backup_cmd")]
    #[test]
    fn test_hash_object_write() {
        let (_td, repo) = repo_init();
        let message = "Hello hash object write!";
        let oid = super::hash_object_write(&repo, message).unwrap();
        let blob = repo.find_blob(oid).unwrap();
        assert_eq!(blob.content(), message.as_bytes());
    }
}
