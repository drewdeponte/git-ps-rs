use git2;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use serde_json;
use super::super::git;

#[derive(Serialize, Deserialize, Debug)]
struct RequestReviewRecord {
    patch_stack_id: Uuid,
    branch_name: String,
    commit_id: String,
    published: Option<bool>,
    location_agnostic_hash: Option<String>
}

pub struct PatchStack<'a> {
    pub head: git2::Reference<'a>,
    pub base: git2::Reference<'a>
}

#[derive(Debug)]
pub enum GetPatchStackError {
    GitError(git2::Error),
    HeadNoName,
    UpstreamBranchNameNotFound
}

impl From<git2::Error> for GetPatchStackError {
    fn from(e: git2::Error) -> Self {
        Self::GitError(e)
    }
}

pub fn get_patch_stack<'a>(repo: &'a git2::Repository) -> Result<PatchStack<'a>, GetPatchStackError> {
    let head_ref = repo.head()?;
    let upstream_branch_name_buf = head_ref.name().ok_or(GetPatchStackError::HeadNoName)
        .and_then(|head_branch_name| repo.branch_upstream_name(head_branch_name).map_err(GetPatchStackError::GitError))?;
    let upstream_branch_name = upstream_branch_name_buf.as_str().ok_or(GetPatchStackError::UpstreamBranchNameNotFound)?;
    let base_ref = repo.find_reference(upstream_branch_name).map_err(GetPatchStackError::GitError)?;

    Ok(PatchStack { head: head_ref, base: base_ref })
}

pub struct ListPatch {
    pub index: usize,
    pub summary: String,
    pub oid: git2::Oid
}

pub fn get_patch_list(repo: &git2::Repository, patch_stack: PatchStack) -> Vec<ListPatch> {
    let mut rev_walk = repo.revwalk().unwrap();
    rev_walk.push(patch_stack.head.target().unwrap()).unwrap();
    rev_walk.hide(patch_stack.base.target().unwrap()).unwrap();
    rev_walk.set_sorting(git2::Sort::REVERSE).unwrap();

    let list_of_patches: Vec<ListPatch> = rev_walk.enumerate().map(|(i, rev)| {
        let r = rev.unwrap();
        ListPatch { index: i, summary: git::get_summary(&repo, &r).unwrap(), oid: r }
    }).collect();
    return list_of_patches;
}

pub fn ls() {
    let repo = match git2::Repository::discover("/Users/adeponte/code/uptech/git-ps") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let path_str = format!("{}{}", repo.path().to_str().unwrap(), "patch-stack-review-requests.json");
    let path = Path::new(&path_str);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => print!("{} contains:\n{}", display, s),
    }

    let rr_records: Vec<RequestReviewRecord> = serde_json::from_str(s.as_str()).unwrap();
    println!("deserialized = {:?}", rr_records);

    let patch_stack = get_patch_stack(&repo).unwrap();
    let list_of_patches = get_patch_list(&repo, patch_stack);

    for patch in list_of_patches.into_iter().rev() {
        println!("{}     {} - {}", patch.index, patch.oid, patch.summary)
    }
}
