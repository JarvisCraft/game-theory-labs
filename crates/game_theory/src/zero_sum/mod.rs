//! [Zero-sum (antagonistic) games][1].
//!
//! [1]: https://en.wikipedia.org/wiki/Zero-sum_game

use std::{fmt, fmt::Formatter};

use nalgebra::{
    allocator::{Allocator, Reallocator},
    ComplexField, DMatrix, DVector, DefaultAllocator, Dim, DimAdd, DimMin, DimMinimum, DimSum, Dyn,
    Matrix, OMatrix, RawStorageMut, SimdPartialOrd, Storage, VecStorage, U1,
};
pub use parse::FromStrError as GameFromStrError;

mod parse;

/// A zeros-sum game defined by its matrix.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Game<M>(pub M);

pub type DGame<T> = Game<DMatrix<T>>;

impl<M: fmt::Display> fmt::Display for Game<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self(matrix) = self;
        matrix.fmt(f)
    }
}

impl<M> Game<M> {
    pub fn new(matrix: M) -> Self {
        Self(matrix)
    }
}

#[allow(type_alias_bounds)] // just for clarity
pub type Strategy<T, N: DimAdd<U1>> = OMatrix<T, DimPlus1<N>, U1>;

impl<T: ComplexField, N: Dim, S: Storage<T, N, N>> Game<Matrix<T, N, N, S>> {
    #[must_use]
    pub fn solve_analytically(&self) -> Option<(Strategy<T, N>, Strategy<T, N>)>
    where
        N: DimAdd<U1>,
        // Define the basic properties of the used dimensions
        DimPlus1<N>: DimMin<DimPlus1<N>, Output = DimPlus1<N>>,
        // the resulting vector will have `N+1` values
        DefaultAllocator: Allocator<T, DimPlus1<N>>
            // we need to add a row of `1`s
            + Reallocator<T, N, N, DimPlus1<N>, N>
            // then we need to add a column of `-1`s
            + Reallocator<T, DimPlus1<N>, N, DimPlus1<N>, DimPlus1<N>>,
    {
        match (
            self.0.transpose().solve_game(),
            self.0.clone_owned().solve_game(),
        ) {
            (Some(a), Some(b)) => Some((a, b)),
            (None, None) => None,
            _ => unreachable!("Either both games are solvable or both games are not solvable"),
        }
    }

    // TODO: maybe, use a more economical return-type of the matrix ones,
    //  since the dimensions are well-defined

    #[must_use]
    pub fn min_win_a(&self) -> DVector<T>
    where
        T: SimdPartialOrd,
    {
        let data: Vec<_> = self.0.row_iter().map(|row| row.min()).collect();
        DVector::from_data(VecStorage::new(Dyn(data.len()), U1, data))
    }

    #[must_use]
    pub fn max_loss_b(&self) -> DVector<T>
    where
        T: SimdPartialOrd,
    {
        let data: Vec<_> = self.0.column_iter().map(|row| row.max()).collect();
        DVector::from_data(VecStorage::new(Dyn(data.len()), U1, data))
    }

    #[must_use]
    pub fn lowest_price(&self) -> (usize, T)
    where
        T: PartialOrd + SimdPartialOrd,
    {
        self.min_win_a().argmax()
    }

    #[must_use]
    pub fn highest_price(&self) -> (usize, T)
    where
        T: PartialOrd + SimdPartialOrd,
    {
        self.max_loss_b().argmin()
    }
}

#[allow(type_alias_bounds)] // just for clarity
type DimPlus1<D: DimAdd<U1>> = DimSum<D, U1>;

pub trait SolveGame {
    type Output;

    fn solve_game(self) -> Option<Self::Output>;
}

impl<T: ComplexField, N: DimAdd<U1>, S: Storage<T, N, N>> SolveGame for Matrix<T, N, N, S>
where
    // Define the basic properties of the used dimensions
    DimPlus1<N>: DimMin<DimPlus1<N>, Output = DimPlus1<N>>,
    // the resulting vector will have `N+1` values
    DefaultAllocator: Allocator<T, DimPlus1<N>>
        // we need to add a row of `1`s
        + Reallocator<T, N, N, DimPlus1<N>, N>
        // then we need to add a column of `-1`s
        + Reallocator<T, DimPlus1<N>, N, DimPlus1<N>, DimPlus1<N>>,
{
    type Output = OMatrix<T, DimPlus1<N>, U1>;

    fn solve_game(self) -> Option<Self::Output> {
        let rows = self.nrows();
        let matrix = self.insert_fixed_rows::<1>(rows, T::one());
        let columns = matrix.ncols();
        let mut matrix = matrix.insert_fixed_columns::<1>(columns, -T::one());
        *matrix
            .iter_mut()
            .last()
            .expect("the matrix should have at least one row and at least one column") = T::zero();
        let a = matrix;

        let n = a.shape_generic().1;
        let mut b = Matrix::zeros_generic(n, U1);
        *b.iter_mut()
            .last()
            .expect("the matrix should have at least one value") = T::one();

        solve::<T, DimMinimum<DimPlus1<N>, DimPlus1<N>>, _, _>(a, b)
    }
}

/// Solves the linear system `a * x = b`, where `x` is the unknown to be determined.
/// This uses the QR decomposition of `A`.
///
/// Returns [`None`] if the system has no solutions.
fn solve<
    T: ComplexField,
    N: Dim,
    SA: Storage<T, N, N>,
    SB: Storage<T, N, U1> + RawStorageMut<T, N, U1>,
>(
    a: Matrix<T, N, N, SA>,
    b: Matrix<T, N, U1, SB>,
) -> Option<Matrix<T, N, U1, SB>>
where
    DefaultAllocator: Allocator<T, N, N> + Allocator<T, N> + Allocator<T, DimMinimum<N, N>>,
    N: DimMin<N, Output = N>,
{
    let a = a.qr();
    let mut b = b;

    a.solve_mut(&mut b).then_some(b)
}
