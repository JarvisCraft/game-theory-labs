use std::fmt::{self, Debug, Display, Formatter};

/// ```
/// fn assert_is_debug<T: std::fmt::Debug>() {}
/// assert_is_debug::<nalgebra::DMatrix<game_theory::non_cooperative::Pair<i32>>>()
/// ```
#[derive(PartialEq, Eq, Hash, Ord, PartialOrd, Clone, Copy)]
pub struct Pair<T>(pub T, pub T);

impl<T: Debug> Debug for Pair<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self(l, r) = self;
        write!(f, "({l:?}, {r:?})")
    }
}

impl<T: Display> Display for Pair<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self(l, r) = self;
        write!(f, "({l}, {r})")
    }
}
