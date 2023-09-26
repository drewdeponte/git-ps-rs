use super::super::super::ps;
use super::git;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct PatchGitInfo {
    pub branches: Vec<ListBranchInfo>,
}

#[derive(Debug)]
pub enum GetListPatchInfoError {
    GetListLocalBranchesWithInfoFailed(GetListLocalBranchesWithInfoError),
}

/// Gets a HashMap of information obtained from Git about the patches, keyed by patch stack id.
///
/// # Arguments
///
/// * `repo` - reference to the repository
/// * `base_oid` - the sha in git2::oid form of the commit at the base of the patch stack
/// * `head_ref_name` - name of HEAD of patch stack branch
pub fn get_list_patch_info(
    repo: &git2::Repository,
    base_oid: git2::Oid,
    head_ref_name: &str,
) -> Result<std::collections::HashMap<Uuid, PatchGitInfo>, GetListPatchInfoError> {
    let mut patch_info_collection: HashMap<Uuid, PatchGitInfo> = HashMap::new();

    let list_branch_info = get_list_local_branches_with_info(repo, base_oid, head_ref_name)
        .map_err(GetListPatchInfoError::GetListLocalBranchesWithInfoFailed)?;

    for bi in list_branch_info {
        for patch_info in bi.patches.iter() {
            if let Some(existing_patch_info) = patch_info_collection.get_mut(&patch_info.patch_id) {
                existing_patch_info.branches.push(bi.clone());
            } else {
                patch_info_collection.insert(
                    patch_info.patch_id,
                    PatchGitInfo {
                        branches: vec![bi.clone()],
                    },
                );
            }
        }
    }

    Ok(patch_info_collection)
}

#[derive(Debug)]
pub enum GetListLocalBranchesWithInfoError {
    GetBranchesFailed(git2::Error),
    GetBranchPairFailed(git2::Error),
    GetListBranchInfoFailed(GetListBranchInfoError),
}

pub fn get_list_local_branches_with_info(
    repo: &git2::Repository,
    base_oid: git2::Oid,
    head_ref_name: &str,
) -> Result<std::vec::Vec<ListBranchInfo>, GetListLocalBranchesWithInfoError> {
    let local_branches: git2::Branches = repo
        .branches(Some(git2::BranchType::Local))
        .map_err(GetListLocalBranchesWithInfoError::GetBranchesFailed)?;

    let mut branch_info_collection: Vec<ListBranchInfo> = Vec::new();

    for branch_pair_result in local_branches {
        let branch_pair =
            branch_pair_result.map_err(GetListLocalBranchesWithInfoError::GetBranchPairFailed)?;
        let branch = branch_pair.0;

        if branch.name().unwrap().unwrap() == head_ref_name {
            continue;
        }

        let branch_info = get_list_branch_info(&branch, base_oid, repo)
            .map_err(GetListLocalBranchesWithInfoError::GetListBranchInfoFailed)?;
        branch_info_collection.push(branch_info);
    }

    Ok(branch_info_collection)
}

#[derive(Debug, Clone)]
pub struct ListBranchInfo {
    pub name: String,
    pub reference: String,
    pub patches: Vec<PatchInfo>,
    pub commit_count: usize,
    pub upstream: Option<ListUpstreamBranchInfo>,
}

#[derive(Debug, Clone)]
pub struct PatchInfo {
    pub patch_id: Uuid,
    pub commit_diff_id: git2::Oid,
    pub commit_timestamp: git2::Time,
}

#[derive(Debug, Clone)]
pub struct ListUpstreamBranchInfo {
    pub name: String,
    pub reference: String,
    pub remote: String,
    pub patches: Vec<PatchInfo>,
    pub commit_count: usize,
}

#[derive(Debug)]
pub enum GetListBranchInfoError {
    GetNameFailed(git2::Error),
    NameInvalidUtf8,
    ReferenceInvalidUtf8,
    RemoteInvalidUtf8,
    GetPatchInfoCollectionFailed(GetPatchInfoCollectionError),
}

pub fn get_list_branch_info(
    branch: &git2::Branch,
    base_oid: git2::Oid,
    repo: &git2::Repository,
) -> Result<ListBranchInfo, GetListBranchInfoError> {
    let name = branch
        .name()
        .map_err(GetListBranchInfoError::GetNameFailed)?
        .ok_or(GetListBranchInfoError::NameInvalidUtf8)?;

    let refname = branch
        .get()
        .name()
        .ok_or(GetListBranchInfoError::ReferenceInvalidUtf8)?;

    let patch_info_collection = get_patch_info_collection(branch, repo, base_oid)
        .map_err(GetListBranchInfoError::GetPatchInfoCollectionFailed)?;

    let upstream_remote_opt = repo.branch_upstream_remote(refname).ok();
    let upstream_branch_opt = branch.upstream().ok();

    let mut upstream_info: Option<ListUpstreamBranchInfo> = None;

    if let (Some(upstream_branch), Some(upstream_remote)) =
        (upstream_branch_opt, upstream_remote_opt)
    {
        let upstream_branch_name = upstream_branch
            .name()
            .map_err(GetListBranchInfoError::GetNameFailed)?
            .ok_or(GetListBranchInfoError::NameInvalidUtf8)?;

        let upstream_branch_refname = upstream_branch
            .get()
            .name()
            .ok_or(GetListBranchInfoError::ReferenceInvalidUtf8)?;

        let upstream_patch_info_collection =
            get_patch_info_collection(&upstream_branch, repo, base_oid)
                .map_err(GetListBranchInfoError::GetPatchInfoCollectionFailed)?;

        upstream_info = Some(ListUpstreamBranchInfo {
            name: upstream_branch_name.to_string(),
            reference: upstream_branch_refname.to_string(),
            remote: upstream_remote
                .as_str()
                .ok_or(GetListBranchInfoError::RemoteInvalidUtf8)?
                .to_string(),
            patches: upstream_patch_info_collection.patch_info_entries,
            commit_count: upstream_patch_info_collection.commit_count,
        })
    }

    Ok(ListBranchInfo {
        name: name.to_string(),
        reference: refname.to_string(),
        patches: patch_info_collection.patch_info_entries,
        commit_count: patch_info_collection.commit_count,
        upstream: upstream_info,
    })
}

#[derive(Debug)]
pub enum GetPatchInfoCollectionError {
    GetBranchHeadOid,
    GetCommonAncestor(git::CommonAncestorError),
    GetCommits(git::GitError),
    GetRevisionOid(git2::Error),
    FindCommit(git2::Error),
    GetCommitDiffPatchId(git::CommitDiffPatchIdError),
}

pub struct PatchInfoCollection {
    pub commit_count: usize,
    pub patch_info_entries: Vec<PatchInfo>,
}

pub fn get_patch_info_collection(
    branch: &git2::Branch,
    repo: &git2::Repository,
    base_oid: git2::Oid,
) -> Result<PatchInfoCollection, GetPatchInfoCollectionError> {
    // go through all the commits between this branch head and common ancestor of the currently
    // checked out branch's upstream branch. e.g. between the common ancestor of origin/main
    // and this branch. I think doing the common ancestor between the currently checked out
    // branch and the current branch would work as well.
    let branch_head_oid = branch
        .get()
        .target()
        .ok_or(GetPatchInfoCollectionError::GetBranchHeadOid)?;
    let common_ancestor_oid = git::common_ancestor(repo, branch_head_oid, base_oid)
        .map_err(GetPatchInfoCollectionError::GetCommonAncestor)?;

    let revwalk = git::get_revs(
        repo,
        common_ancestor_oid,
        branch_head_oid,
        git2::Sort::REVERSE,
    )
    .map_err(GetPatchInfoCollectionError::GetCommits)?;

    let mut patch_info_entries: Vec<PatchInfo> = Vec::new();
    let mut commit_count: usize = 0;

    for oid_result in revwalk {
        let oid = oid_result.map_err(GetPatchInfoCollectionError::GetRevisionOid)?;
        commit_count += 1;
        let commit = repo
            .find_commit(oid)
            .map_err(GetPatchInfoCollectionError::FindCommit)?;

        if let Some(ps_id) = ps::commit_ps_id(&commit) {
            let commit_diff_id = git::commit_diff_patch_id(repo, &commit)
                .map_err(GetPatchInfoCollectionError::GetCommitDiffPatchId)?;

            patch_info_entries.push(PatchInfo {
                patch_id: ps_id,
                commit_timestamp: commit.time(),
                commit_diff_id,
            });
        }
    }

    Ok(PatchInfoCollection {
        commit_count,
        patch_info_entries,
    })
}
