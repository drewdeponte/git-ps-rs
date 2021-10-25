// This is the `test` module. It is an internal module that is only loaded
// for test compilations. It is resposible for housing any custom helper
// functionality use to make writing tests easier. This will most likely
// consist of functions to help with test setup or functions to assert with
// common types of assertions.

use tempfile::TempDir;
use git2::{Repository, RepositoryInitOptions};

pub fn repo_init() -> (TempDir, Repository) {
    let td = TempDir::new().unwrap();
    let mut opts = RepositoryInitOptions::new();
    opts.initial_head("main");
    let repo = Repository::init_opts(td.path(), &opts).unwrap();
    {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "name").unwrap();
        config.set_str("user.email", "email").unwrap();
        let mut index = repo.index().unwrap();
        let id = index.write_tree().unwrap();

        let tree = repo.find_tree(id).unwrap();
        let sig = repo.signature().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();
    }
    (td, repo)
}

pub fn create_commit(repo: &git2::Repository, path: &str, data: &[u8], message: &str) -> git2::Oid {
  // To implement this I was losely following
  // https://stackoverflow.com/questions/15711444/how-to-commit-to-a-git-repository-using-libgit2
  let sig = git2::Signature::now("Bob Villa", "bob@example.com").unwrap();

  // create the blob record for storing the content
  let blob_oid = repo.blob(data).unwrap();
  // repo.find_blob(blob_oid).unwrap();

  // create the tree record
  let mut treebuilder = repo.treebuilder(Option::None).unwrap();
  let file_mode: i32 = i32::from(git2::FileMode::Blob);
  treebuilder.insert(path, blob_oid, file_mode).unwrap();
  let tree_oid = treebuilder.write().unwrap();

  // lookup the tree entity
  let tree = repo.find_tree(tree_oid).unwrap();

  // TODO: need to figure out some way to get the parent commit as a
  // git2::Commit object to hand
  // into the repo.commit call. I am guessing that is why I am getting
  // the following error
  // "failed to create commit: current tip is not the first parent"
  let parent_oid = repo.head().unwrap().target().unwrap();
  let parent_commit = repo.find_commit(parent_oid).unwrap();

  // create the actual commit packaging the blob, tree entry, etc.
  repo.commit(Option::Some("HEAD"), &sig, &sig, message, &tree, &[&parent_commit]).unwrap()
}
