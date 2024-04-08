use std::fmt::{self, Debug, Display, Formatter};

use nalgebra::{DMatrix, Dyn, VecStorage};
use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    Rng,
};

pub fn random_matrix<T: SampleUniform>(
    mut random: impl Rng,
    rows: usize,
    columns: usize,
    range: impl SampleRange<T> + Clone,
) -> DMatrix<T> {
    DMatrix::from_vec_storage(VecStorage::new(
        Dyn(rows),
        Dyn(columns),
        (0..rows * columns)
            .map(|_| random.gen_range(range.clone()))
            .collect(),
    ))
}

pub fn random_bi_matrix<T: SampleUniform>(
    mut random: impl Rng,
    rows: usize,
    columns: usize,
    range: impl SampleRange<T> + Clone,
) -> DMatrix<Pair<T>> {
    DMatrix::from_vec_storage(VecStorage::new(
        Dyn(rows),
        Dyn(columns),
        (0..rows * columns)
            .map(|_| {
                Pair(
                    random.gen_range(range.clone()),
                    random.gen_range(range.clone()),
                )
            })
            .collect(),
    ))
}

/// ```
/// fn assert_is_debug<T: std::fmt::Debug>() {}
/// assert_is_debug::<nalgebra::DMatrix<game_theory::generate::Pair<i32>>>()
/// ```
#[derive(PartialEq, Eq, Hash, Ord, PartialOrd, Clone)]
pub struct Pair<T>(T, T);

impl<T: Debug> Debug for Pair<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self(l, r) = self;
        write!(f, "[{l:?}, {r:?}]")
    }
}

impl<T: Display> Display for Pair<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self(l, r) = self;
        write!(f, "[{l}, {r}]")
    }
}
