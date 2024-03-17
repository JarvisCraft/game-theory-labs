//! Implementation of the Brown-Robinson method.

use nalgebra::{
    allocator::Allocator, DefaultAllocator, Dim, Matrix, OMatrix, Scalar, SimdPartialOrd, Storage,
    U1,
};
use num_traits::{float::FloatCore, Zero};
use ordered_float::NotNan;
use rand::{Rng, thread_rng};

use game_theory::zero_sum::Game;

mod iter;

// TODO: get rid of the exact used type
type Value = f64;

pub struct BrownRobinsonRow<T, N: Dim>
where
    DefaultAllocator: Allocator<T, U1, N>,
{
    /// Номер текущей итерации
    pub iteration: usize,
    /// Текущая стратегия игрока A
    pub a_strategy: usize,
    /// Текущая стратегия игрока B
    pub b_strategy: usize,
    /// Накопленный выигрыш игрока A
    pub a_score: OMatrix<T, U1, N>,
    /// Накопленный выигрыш игрока B
    pub b_score: OMatrix<T, U1, N>,
    /// Верхняя цена игры
    pub high_price: T,
    /// Нижняя цена игры
    pub low_price: T,
    /// ε, разница между минимальной верхней и максиммальной нижней ценами игры
    pub epsilon: T,
}

// Итератор по шагам метода
pub struct BrownRobinson<T, N: Dim, S: Storage<T, N, N>>
where
    DefaultAllocator: Allocator<usize, U1, N> + Allocator<T, U1, N>,
{
    game: Game<Matrix<T, N, N, S>>,
    a_strategy: usize,
    b_strategy: usize,
    a_scores: OMatrix<T, U1, N>,
    b_scores: OMatrix<T, U1, N>,
    min_high_price: T,
    max_low_price: T,
    a_strategy_times_used: OMatrix<usize, U1, N>,
    b_strategy_times_used: OMatrix<usize, U1, N>,
    /// The number of the current iteration.
    k: usize,
}

impl<T: Scalar + Zero + SimdPartialOrd, N: Dim, S: Storage<T, N, N>> BrownRobinson<T, N, S>
where
    DefaultAllocator: Allocator<usize, U1, N> + Allocator<T, U1, N>,
{
    #[must_use]
    pub fn new(game_matrix: Matrix<T, N, N, S>) -> Self {
        let a_strategy = thread_rng().gen_range(0..game_matrix.nrows());
        let b_strategy = thread_rng().gen_range(0..game_matrix.ncols());

        let a_scores = game_matrix.row(a_strategy).clone_owned();
        let b_scores = game_matrix.column(b_strategy).transpose().clone_owned();
        let min_high_price = a_scores.max();
        let max_low_price = b_scores.min();

        let mut a_strategy_times_used = Matrix::zeros_generic(U1, game_matrix.shape_generic().0);
        a_strategy_times_used[a_strategy] = 1;
        let mut b_strategy_times_used = Matrix::zeros_generic(U1, game_matrix.shape_generic().1);
        b_strategy_times_used[b_strategy] = 1;

        Self {
            game: Game::new(game_matrix),
            a_strategy,
            b_strategy,
            a_scores,
            b_scores,
            min_high_price,
            max_low_price,
            a_strategy_times_used,
            b_strategy_times_used,
            k: 0,
        }
    }

    #[must_use]
    pub fn bounds(&self) -> (T, T)
    where
        T: FloatCore,
    {
        let max_min = self
            .game
            .0
            .column_iter()
            .map(|row| NotNan::new(row.min()).unwrap())
            .max()
            .unwrap();
        let min_max = self
            .game
            .0
            .row_iter()
            .map(|row| NotNan::new(row.max()).unwrap())
            .min()
            .unwrap();

        // TODO: preserve lifetime of `self` when generics
        (*max_min, *min_max)
    }

    #[must_use]
    pub fn game(&self) -> &Game<Matrix<T, N, N, S>> {
        &self.game
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
    pub fn strategies_used(&self) -> (&OMatrix<usize, U1, N>, &OMatrix<usize, U1, N>) {
        (&self.a_strategy_times_used, &self.b_strategy_times_used)
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
