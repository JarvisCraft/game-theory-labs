//! Implementation of the Brown-Robinson method.

mod iter;

use nalgebra::{
    one, ArrayStorage, ComplexField, Matrix1x3, Matrix1x4, Matrix3, Matrix4x1, Scalar,
    SimdPartialOrd,
};
use num_traits::{float::FloatCore, Zero};
use ordered_float::NotNan;
use rand::{thread_rng, Rng};

const M: usize = 3;
const N: usize = 3;

// TODO: get rid of hardcode
type Value = f64;

pub struct BrownRobinsonRow<T> {
    /// Номер текущей итерации
    pub iteration: usize,
    /// Текущая стратегия игрока A
    pub a_strategy: usize,
    /// Текущая стратегия игрока B
    pub b_strategy: usize,
    /// Накопленный выигрыш игрока A
    pub a_score: Matrix1x3<T>,
    /// Накопленный выигрыш игрока B
    pub b_score: Matrix1x3<T>,
    /// Верхняя цена игры
    pub high_price: T,
    /// Нижняя цена игры
    pub low_price: T,
    /// ε, разница между минимальной верхней и максиммальной нижней ценами игры
    pub epsilon: T,
}

// Итератор по шагам метода
pub struct BrownRobinson<T> {
    game_matrix: Matrix3<T>,
    a_strategy: usize,
    b_strategy: usize,
    a_scores: Matrix1x3<T>,
    b_scores: Matrix1x3<T>,
    min_high_price: T,
    max_low_price: T,
    a_strategy_used: [usize; M],
    b_strategy_used: [usize; N],
    k: usize,
}

impl<T: Scalar + Zero + SimdPartialOrd> BrownRobinson<T> {
    #[must_use]
    pub fn new(game_matrix: [[T; M]; N]) -> Self {
        let game_matrix = Matrix3::from_data(ArrayStorage(game_matrix));
        let a_strategy = thread_rng().gen_range(0..M);
        let b_strategy = thread_rng().gen_range(0..N);

        let a_scores = game_matrix.row(a_strategy).clone_owned();
        let b_scores = game_matrix.column(b_strategy).transpose().clone_owned();
        let min_high_price = a_scores.max();
        let max_low_price = b_scores.min();

        let mut a_strategy_used = [0; M];
        a_strategy_used[a_strategy] = 1;
        let mut b_strategy_used = [0; N];
        b_strategy_used[b_strategy] = 1;

        Self {
            game_matrix,
            a_strategy,
            b_strategy,
            a_scores,
            b_scores,
            min_high_price,
            max_low_price,
            a_strategy_used,
            b_strategy_used,
            k: 0,
        }
    }

    #[must_use]
    pub fn bounds(&self) -> (T, T)
    where
        T: FloatCore,
    {
        let max_min = self
            .game_matrix
            .column_iter()
            .map(|row| NotNan::new(row.min()).unwrap())
            .max()
            .unwrap();
        let min_max = self
            .game_matrix
            .row_iter()
            .map(|row| NotNan::new(row.max()).unwrap())
            .min()
            .unwrap();

        // TODO: preserve lifetime of `self` when generics
        (*max_min, *min_max)
    }

    #[must_use]
    pub fn game_matrix(&self) -> &Matrix3<T> {
        &self.game_matrix
    }

    #[must_use]
    pub fn min_max_prices(&self) -> (&T, &T) {
        (&self.max_low_price, &self.min_high_price)
    }

    #[must_use]
    pub fn k(&self) -> usize {
        self.k
    }

    #[must_use]
    pub fn strategies_used(&self) -> ([usize; M], [usize; N]) {
        (self.a_strategy_used, self.b_strategy_used)
    }

    #[must_use]
    fn high_price(&self) -> T
    where
        T: SimdPartialOrd + Zero,
    {
        self.a_scores.max()
    }

    #[must_use]
    fn low_price(&self) -> T
    where
        T: SimdPartialOrd + Zero,
    {
        self.b_scores.min()
    }
}

// TODO: encapsulate
pub fn solve_linear_equations<T: ComplexField>(matrix: Matrix3<T>) -> Matrix1x4<T> {
    // FIXME: use relative addressing and zero assignment zero
    let matrix = matrix.insert_fixed_rows::<1>(3, one());
    let mut matrix = matrix.insert_fixed_columns::<1>(3, -T::one());
    // FIXME: use relative addressing and zero assignment zero
    matrix[(3, 3)] = T::zero();
    let a = matrix;
    println!("{a}");

    // let row_0 = matrix.row(0).data.to_owned();
    // let row_1 = matrix.row(1).data.to_owned();
    // let row_2 = matrix.row(2).data.to_owned();
    // let a = Matrix4::from_data(ArrayStorage([
    //     [row_0[0], row_0[1], row_0[2], -T::one()],
    //     [row_1[0], row_1[1], row_1[2], -T::one()],
    //     [row_2[0], row_2[1], row_2[2], -T::one()],
    //     [T::one(), T::one(), T::one(), T::zero()],
    // ]));
    let b = Matrix4x1::from_data(ArrayStorage([[T::zero(), T::zero(), T::zero(), T::one()]]));

    a.lu().solve(&b).unwrap().transpose()
}
