use super::super::private::config;
use super::super::private::git;
use super::super::private::paths;
use ansi_term::Colour::{Cyan, Yellow};

#[derive(Debug)]
pub enum UpstreamPatchesError {
    RepositoryMissing,
    GetRepoRootPathFailed(paths::PathsError),
    PathNotUtf8,
    GetConfigFailed(config::GetConfigError),
    GetHeadRefFailed,
    GetHeadRefTargetFailed,
    GetHeadBranchNameFailed,
    GetUpstreamBranchNameFailed,
    FindUpstreamBranchReferenceFailed(git2::Error),
    GetUpstreamBranchOidFailed,
}

impl std::fmt::Display for UpstreamPatchesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepositoryMissing => write!(f, "repository missing"),
            Self::GetRepoRootPathFailed(e) => write!(f, "get repository root path failed, {}", e),
            Self::PathNotUtf8 => write!(f, "path not utf-8"),
            Self::GetConfigFailed(e) => write!(f, "get config failed, {}", e),
            Self::GetHeadRefFailed => write!(f, "get head reference failed"),
            Self::GetHeadRefTargetFailed => write!(f, "get head reference target failed"),
            Self::GetHeadBranchNameFailed => write!(f, "get head branch name failed"),
            Self::GetUpstreamBranchNameFailed => write!(f, "get upstream branch name failed"),
            Self::FindUpstreamBranchReferenceFailed(e) => {
                write!(f, "find upstream branch reference failed, {}", e)
            }
            Self::GetUpstreamBranchOidFailed => write!(f, "get upstream branch oid failed"),
        }
    }
}

impl std::error::Error for UpstreamPatchesError {}

pub fn upstream_patches(color: bool) -> Result<(), UpstreamPatchesError> {
    let repo = git::create_cwd_repo().map_err(|_| UpstreamPatchesError::RepositoryMissing)?;

    // get the start & end oids - e.g. origin/main & main
    let head_ref = repo
        .head()
        .map_err(|_| UpstreamPatchesError::GetHeadRefFailed)?;
    let head_oid = head_ref
        .target()
        .ok_or(UpstreamPatchesError::GetHeadRefTargetFailed)?;

    let head_branch_name = head_ref
        .name()
        .ok_or(UpstreamPatchesError::GetHeadBranchNameFailed)?;
    let upstream_branch_name = git::branch_upstream_name(&repo, head_branch_name)
        .map_err(|_| UpstreamPatchesError::GetUpstreamBranchNameFailed)?;

    let upstream_branch_ref = repo
        .find_reference(&upstream_branch_name)
        .map_err(UpstreamPatchesError::FindUpstreamBranchReferenceFailed)?;
    let upstream_branch_oid = upstream_branch_ref
        .target()
        .ok_or(UpstreamPatchesError::GetUpstreamBranchOidFailed)?;

    let mut rev_walk = git::get_revs(
        &repo,
        head_oid,
        upstream_branch_oid,
        git2::Sort::TOPOLOGICAL,
    )
    .unwrap()
    .peekable();
    println!();
    if rev_walk.peek().is_none() {
        // empty
        println!("None. You are already up to date!");
    }
    rev_walk.for_each(|oid| {
        let sha = oid.unwrap();
        let patch_oid_str = format!("{:.6}", sha);
        let commit = repo.find_commit(sha).unwrap();
        let summary = commit.summary().unwrap_or("").to_string();
        let signature = commit.author();
        let display_name = signature
            .name()
            .unwrap_or_else(|| signature.email().unwrap_or("Unknown"));

        if color {
            println!(
                "* {:.6} {} {}",
                Yellow.paint(patch_oid_str),
                summary,
                Cyan.paint(display_name)
            );
        } else {
            println!("* {:.6} {} {}", patch_oid_str, summary, signature);
        }
    });

    Ok(())
}
