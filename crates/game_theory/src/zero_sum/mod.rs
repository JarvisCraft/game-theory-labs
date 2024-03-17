//! [Zero-sum (antagonistic) games][1].
//!
//! [1]: https://en.wikipedia.org/wiki/Zero-sum_game

use std::{fmt, fmt::Formatter};

use nalgebra::{
    allocator::{Allocator, Reallocator},
    ComplexField, DefaultAllocator, Dim, DimAdd, DimMin, DimMinimum, DimSum, Matrix, Matrix3,
    OMatrix, RawStorageMut, Scalar, Storage, U1,
};
use num_traits::{One, Zero};

/// A zeros-sum game defined by its matrix.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Game<M>(pub M);

impl<M: fmt::Display> fmt::Display for Game<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self(matrix) = self;
        write!(f, "{matrix}")
    }
}

impl<M> Game<M> {
    pub fn new(matrix: M) -> Self {
        Self(matrix)
    }
}

impl<T: Scalar, D: Dim, S: Storage<T, D, D>> Game<Matrix<T, D, D, S>> {
    // pub fn analytical_strategies() -> (Matrix<T, U1, DimSum<C, >, >)
    // where T: ComplexField {
    //     x: DMatrix
    // }

    pub fn a_strategy(&self) -> OMatrix<T, U1, DimPlus1<D>>
    where
        D: DimAdd<U1>,
        DefaultAllocator: Allocator<T, U1, DimPlus1<D>>,
        //     T: ComplexField,
        //     C: DimAdd<U1>,
        //     DefaultAllocator: Allocator<T, U1, <C as DimAdd<U1>>::Output>,
        //     <C as DimAdd<U1>>::Output: DimAdd<U1>,
    {
        todo!()
        // solve_linear_equations_analytically(self.0.clone_owned())
    }
}

#[allow(type_alias_bounds)] // just for clarity
type DimPlus1<D: DimAdd<U1>> = DimSum<D, U1>;

pub fn solve_linear_equations_analytically<T: ComplexField, D: DimAdd<U1>, S: Storage<T, D, D>>(
    matrix: Matrix<T, D, D, S>,
) -> Option<OMatrix<T, DimPlus1<D>, U1>>
where
    // Define the basic properties of the used dimensions
    DimPlus1<D>: DimMin<DimPlus1<D>, Output = DimPlus1<D>>,
    // the resulting vector will have C+1 values
    DefaultAllocator: Allocator<T, DimPlus1<D>>,
    DefaultAllocator: Reallocator<T, D, D, D, DimPlus1<D>>,
    // we need to add a row of `1`s
    DefaultAllocator: Allocator<T, DimPlus1<D>, D>,
    DefaultAllocator: Reallocator<T, D, D, DimPlus1<D>, D>,
    // then we need to add a column of `-1`s
    DefaultAllocator: Allocator<T, DimPlus1<D>, DimPlus1<D>>,
    DefaultAllocator: Reallocator<T, DimPlus1<D>, D, DimPlus1<D>, DimPlus1<D>>,
    // finally, we need to solve the equation in-place
    DefaultAllocator: Allocator<T, DimPlus1<D>>,
{
    let rows = matrix.nrows();
    let matrix = matrix.insert_fixed_rows::<1>(rows, T::one());
    let columns = matrix.ncols();
    let mut matrix = matrix.insert_fixed_columns::<1>(columns, -T::one());
    *matrix.as_mut_slice().last_mut().unwrap() = T::zero();
    let a = matrix;

    let n = a.shape_generic().1;
    solve::<T, DimMinimum<DimPlus1<D>, DimPlus1<D>>, _, _>(a, Matrix::zeros_generic(n, U1))
}

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

pub trait SolveAnalytically<T: Scalar, D: DimAdd<U1>>
where
    DefaultAllocator: Allocator<T, DimPlus1<D>, U1>,
{
    fn solve_a(&self) -> OMatrix<T, DimPlus1<D>, U1>;
}

impl<T: Scalar + Zero + One + ComplexField> SolveAnalytically<T, nalgebra::U3>
    for Game<Matrix3<T>>
{
    fn solve_a(&self) -> nalgebra::Matrix4x1<T> {
        let matrix = self.0.clone();

        let rows = matrix.nrows();
        let matrix = matrix.insert_fixed_rows::<1>(rows, T::one());
        let columns = matrix.ncols();
        let mut matrix = matrix.insert_fixed_columns::<1>(columns, -T::one());
        *matrix.as_mut_slice().last_mut().unwrap() = T::zero();
        let a = matrix;

        let mut b: Matrix<T, nalgebra::U4, U1, _> = Zero::zero();
        *b.as_mut_slice().last_mut().unwrap() = T::one();

        let a = a.qr();
        a.solve(&b).unwrap()
    }
}
