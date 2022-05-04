pub trait Mergable {
  fn merge(&self, b: &Self) -> Self;
}
