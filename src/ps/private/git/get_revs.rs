use super::git_error::GitError;
use git2;
use std::result::Result;

/// Attempt to get revs given a repo, start Oid (excluded), and end Oid (included)
pub fn get_revs(
    repo: &git2::Repository,
    start: git2::Oid,
    end: git2::Oid,
    sort: git2::Sort,
) -> Result<git2::Revwalk, GitError> {
    let mut rev_walk = repo.revwalk()?;
    rev_walk.push(end)?;
    rev_walk.hide(start)?;
    rev_walk.set_sorting(sort)?;
    Ok(rev_walk)
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::{create_commit, repo_init};

    #[test]
    fn smoke_get_revs() {
        let (_td, repo) = repo_init();

        let start_oid_excluded =
            create_commit(&repo, "fileA.txt", &[0, 1, 2, 3], "starting numbers");
        create_commit(
            &repo,
            "fileB.txt",
            &[4, 5, 6, 7],
            "four, five, six, and seven",
        );
        let end_oid_included = create_commit(
            &repo,
            "fileC.txt",
            &[8, 9, 10, 11],
            "eight, nine, ten, and eleven",
        );
        create_commit(
            &repo,
            "fileD.txt",
            &[12, 13, 14, 15],
            "twelve, thirteen, forteen, fifteen",
        );

        let rev_walk = super::get_revs(
            &repo,
            start_oid_excluded,
            end_oid_included,
            git2::Sort::REVERSE,
        )
        .unwrap();
        let summaries: Vec<String> = rev_walk
            .map(|oid| {
                repo.find_commit(oid.unwrap())
                    .unwrap()
                    .summary()
                    .unwrap()
                    .to_string()
            })
            .collect();
        assert_eq!(summaries.len(), 2);

        assert_eq!(summaries.first().unwrap(), "four, five, six, and seven");
        assert_eq!(summaries.last().unwrap(), "eight, nine, ten, and eleven");
    }
}
