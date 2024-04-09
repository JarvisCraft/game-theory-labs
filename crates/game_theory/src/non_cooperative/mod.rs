use std::{fmt, fmt::Formatter};

use nalgebra::{ComplexField, DMatrix, Dyn, VecStorage};
pub use pair::Pair;
use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    Rng,
};

mod optimal;
mod pair;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Game<G>(pub G);

impl<G> Game<G> {
    pub fn new(game: G) -> Self {
        Self(game)
    }
}

impl<M: fmt::Display> fmt::Display for Game<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self(matrix) = self;
        matrix.fmt(f)
    }
}

pub type BiMatrixGame<T> = Game<DMatrix<Pair<T>>>;

pub use optimal::OptimalBiMatrixStrategy;

impl<T> BiMatrixGame<T> {
    pub fn random<G>(
        mut random: impl Rng,
        rows: usize,
        columns: usize,
        range: impl SampleRange<G> + Clone,
        mapper: impl Fn(G) -> T,
    ) -> Self
    where
        G: SampleUniform,
    {
        Self(DMatrix::from_vec_storage(VecStorage::new(
            Dyn(rows),
            Dyn(columns),
            (0..rows * columns)
                .map(|_| {
                    Pair(
                        mapper(random.gen_range(range.clone())),
                        mapper(random.gen_range(range.clone())),
                    )
                })
                .collect(),
        )))
    }

    pub fn mixed_balanced_strategies(&self) -> Option<((T, T), (DMatrix<T>, DMatrix<T>))>
    where
        T: ComplexField + Copy,
    {
        if self.0.is_empty() {
            return None;
        }

        let (a, b) = self.split();
        let a_inv = a.lu().try_inverse()?;
        let b_inv = b.lu().try_inverse()?;

        let v1 = T::one()
            / (DMatrix::repeat(1, a_inv.nrows(), T::one())
                * a_inv.clone()
                * DMatrix::repeat(a_inv.ncols(), 1, T::one()))[(0, 0)];
        let v2 = T::one()
            / (DMatrix::repeat(1, b_inv.nrows(), T::one())
                * b_inv.clone()
                * DMatrix::repeat(b_inv.ncols(), 1, T::one()))[(0, 0)];

        let b_inv_rows = b_inv.nrows();
        let mut x = DMatrix::repeat(1, b_inv_rows, T::one()) * b_inv;
        for element in &mut x {
            *element *= v2;
        }

        let a_inv_cols = a_inv.ncols();
        let mut y = a_inv * DMatrix::repeat(a_inv_cols, 1, T::one());
        for element in &mut y {
            *element *= v1;
        }
        Some(((v1, v2), (x, y.transpose())))
    }

    fn split(&self) -> (DMatrix<T>, DMatrix<T>)
    where
        T: Clone,
    {
        let Self(game) = self;

        let (rows, columns) = (game.nrows(), game.ncols());
        let capacity = rows * columns;

        let (mut a_elements, mut b_elements) =
            (Vec::with_capacity(capacity), Vec::with_capacity(capacity));
        for Pair(a, b) in game.iter().cloned() {
            a_elements.push(a);
            b_elements.push(b);
        }

        (
            DMatrix::from_vec_storage(VecStorage::new(Dyn(rows), Dyn(columns), a_elements)),
            DMatrix::from_vec_storage(VecStorage::new(Dyn(rows), Dyn(columns), b_elements)),
        )
    }
}
