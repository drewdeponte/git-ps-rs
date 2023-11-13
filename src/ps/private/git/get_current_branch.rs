use git2;

pub fn get_current_branch(repo: &git2::Repository) -> Option<String> {
    // https://stackoverflow.com/questions/12132862/how-do-i-get-the-name-of-the-current-branch-in-libgit2
    match repo.head() {
        Ok(head_ref) => return head_ref.name().map(String::from),
        Err(_) => None,
    }
}
