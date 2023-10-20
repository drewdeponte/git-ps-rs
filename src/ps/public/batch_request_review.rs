use super::request_review;
use std::result::Result;

#[derive(Debug)]
pub enum BatchRequestReviewError {
    RequestReviewErrors(Vec<request_review::RequestReviewError>),
}

pub fn batch_request_review(
    patch_indexes: Vec<usize>,
    color: bool,
) -> Result<(), BatchRequestReviewError> {
    let mut errors: Vec<request_review::RequestReviewError> = Vec::new();
    for patch_index in patch_indexes {
        match request_review::request_review(patch_index, None, None, color, true, true) {
            Ok(_) => {}
            Err(e) => errors.push(e),
        };
    }
    match errors.len() {
        0 => Ok(()),
        _ => Err(BatchRequestReviewError::RequestReviewErrors(errors)),
    }
}
