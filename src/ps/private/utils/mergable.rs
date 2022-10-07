pub trait Mergable {
  fn merge(&self, b: &Self) -> Self;
}

/// Merge optional mergables together given mergable a & b where b takes presedence
pub fn merge_option<T: Mergable + std::clone::Clone>(a: &Option<T>, b: &Option<T>) -> Option<T> {
  b.as_ref().map(|bb| a.as_ref().map(|aa| aa.merge(bb)).unwrap_or_else(|| (*bb).clone()) ).or_else(|| a.clone())
}
