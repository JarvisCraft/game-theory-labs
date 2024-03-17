//! Implementation of the Brown-Robinson method.

use nalgebra::{
    ArrayStorage, ComplexField, Matrix1x3, Matrix1x4, Matrix3, Matrix4x1, one, Scalar,
    SimdPartialOrd,
};
use num_traits::{float::FloatCore, Zero};
use ordered_float::NotNan;
use rand::{Rng, thread_rng};

use game_theory::zero_sum::Game;

mod iter;

const M: usize = 3;
const N: usize = 3;

// TODO: get rid of the exact used type
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
    game: Game<Matrix3<T>>,
    a_strategy: usize,
    b_strategy: usize,
    a_scores: Matrix1x3<T>,
    b_scores: Matrix1x3<T>,
    min_high_price: T,
    max_low_price: T,
    a_strategy_used: [usize; M],
    b_strategy_used: [usize; N],
    /// The number of the current iteration.
    k: usize,
}

impl<T: Scalar + Zero + SimdPartialOrd> BrownRobinson<T> {
    #[must_use]
    pub fn new(game_matrix: Matrix3<T>) -> Self {
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
            game: Game::new(game_matrix),
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
    pub fn game(&self) -> &Game<Matrix3<T>> {
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
