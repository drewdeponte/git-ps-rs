use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;
use std::result::Result;
use uuid::Uuid;
use super::paths;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PatchState {
  BranchCreated(String, String), // branch_name, diff_hash
  PushedToRemote(String, String, String), // remote, branch_name, diff_hash
  RequestedReview(String, String, String), // remote, branch_name, diff_hash
  Integrated(String, String, String) // remote, branch_name, diff_hash
}

impl PatchState {
  pub fn branch_name(&self) -> String {
    match self {
      Self::BranchCreated(branch_name, _) => branch_name.to_string(),
      Self::PushedToRemote(_, branch_name, _) => branch_name.to_string(),
      Self::RequestedReview(_, branch_name, _) => branch_name.to_string(),
      Self::Integrated(_, branch_name, _) => branch_name.to_string()
    }
  }

  pub fn has_been_pushed_to_remote(&self) -> bool {
    match self {
      Self::BranchCreated(_, _) => false,
      Self::PushedToRemote(_, _, _) => true,
      Self::RequestedReview(_, _, _) => true,
      Self::Integrated(_, _, _) => false
    }
  }

  pub fn has_requested_review(&self) -> bool {
    match self {
      Self::BranchCreated(_, _) => false,
      Self::PushedToRemote(_, _, _) => false,
      Self::RequestedReview(_, _, _) => true,
      Self::Integrated(_, _, _) => true
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Patch {
  pub patch_id: Uuid,
  pub state: PatchState
}

#[derive(Debug)]
pub enum ReadPatchStatesError {
  OpenFailed(io::Error),
  ReadOrDeserializeFailed(serde_json::Error)
}

pub fn read_patch_states<P: AsRef<Path>>(path: P) -> Result<HashMap<Uuid, Patch>, ReadPatchStatesError> {
  match File::open(path) {
    Err(e) => {
      match e.kind() {
        io::ErrorKind::NotFound => Ok(HashMap::new()),
        _ => Err(ReadPatchStatesError::OpenFailed(e))
      }
    },
    Ok(file) => {
      let reader = io::BufReader::new(file);
      let patch_states = serde_json::from_reader(reader).map_err(|e| ReadPatchStatesError::ReadOrDeserializeFailed(e))?;
      Ok(patch_states)
    }
  }
}

#[derive(Debug)]
pub enum WritePatchStatesError {
  OpenFailed(io::Error),
  WriteOrSerializeFailed(serde_json::Error)
}

pub fn write_patch_states(path: &Path, patch_state: &HashMap<Uuid, Patch>) -> Result<(), WritePatchStatesError> {
  let file = File::create(path).map_err(|e| WritePatchStatesError::OpenFailed(e))?;
  let writer = io::BufWriter::new(file);
  serde_json::to_writer_pretty(writer, patch_state).map_err(|e| WritePatchStatesError::WriteOrSerializeFailed(e))?;
  Ok(())
}

#[derive(Debug)]
pub enum StorePatchStateError {
  PatchStatesPathNotFound,
  ReadPatchStatesFailed(ReadPatchStatesError),
  WritePatchStatesFailed(WritePatchStatesError)
}

pub fn store_patch_state(repo: &git2::Repository, patch_state: &Patch) -> Result<(), StorePatchStateError> {
  // get path to patch states file
  let states_path = paths::patch_states_path(repo)
    .map_err(|_| StorePatchStateError::PatchStatesPathNotFound)?;

  // read the patch states in
  // let mut patch_states: HashMap<Uuid, Patch> = read_patch_states(
  let mut patch_states = read_patch_states(&states_path)
    .map_err(|e| StorePatchStateError::ReadPatchStatesFailed(e))?;
  
  // add the patch to the hash map
  let patch_state_copy: Patch = patch_state.clone();
  patch_states.insert(patch_state.patch_id, patch_state_copy);
  
  // write the patch states out
  write_patch_states(&states_path, &patch_states)
    .map_err(|e| StorePatchStateError::WritePatchStatesFailed(e))?;

  Ok(())
}

pub fn update_patch_state(repo: &git2::Repository, patch_id: &Uuid, f: impl FnOnce(Option<Patch>) -> Patch) -> Result<(), StorePatchStateError> {
  // get path to patch states file
  let states_path = paths::patch_states_path(repo)
    .map_err(|_| StorePatchStateError::PatchStatesPathNotFound)?;

  // read the patch states in
  // let mut patch_states: HashMap<Uuid, Patch> = read_patch_states(
  let mut patch_states = read_patch_states(&states_path)
    .map_err(|e| StorePatchStateError::ReadPatchStatesFailed(e))?;

  // add the patch to the hash map
  let patch_state_copy: Patch = f(patch_states.get(patch_id).cloned());
  patch_states.insert(patch_state_copy.patch_id, patch_state_copy);

  // write the patch states out
  write_patch_states(&states_path, &patch_states)
    .map_err(|e| StorePatchStateError::WritePatchStatesFailed(e))?;

  Ok(())
}

#[derive(Debug)]
pub enum FetchPatchMetaDataError {
  FailedToGetPathToMetaData(paths::PathsError),
  FailedToReadMetaData(ReadPatchStatesError)
}

pub fn fetch_patch_meta_data(repo: &git2::Repository, patch_id: &Uuid) -> Result<Option<Patch>, FetchPatchMetaDataError> {
  let patch_meta_data_path = paths::patch_states_path(repo).map_err(|e| FetchPatchMetaDataError::FailedToGetPathToMetaData(e))?;
  let patch_meta_data = read_patch_states(patch_meta_data_path).map_err(|e| FetchPatchMetaDataError::FailedToReadMetaData(e))?;
  Ok(patch_meta_data.get(patch_id).cloned())
}
