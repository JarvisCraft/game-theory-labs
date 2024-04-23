pub mod backward_induction;
// pub mod tree;

/// A positional game defined by its tree.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Game<T>(pub T);
