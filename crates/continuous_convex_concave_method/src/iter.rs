use std::{collections::VecDeque, iter::FusedIterator, num::NonZeroUsize};

use brown_robinson_method::{BrownRobinson, BrownRobinsonRow};
use game_theory::zero_sum::Game;
use nalgebra::{ComplexField, DMatrix, Dyn, VecStorage};
use tracing::{debug, span, trace, Level};

use crate::{ContinuousConvexConcaveGame, GameSolution};

pub struct Iter<'a, T> {
    /// The iterated game
    game: &'a ContinuousConvexConcaveGame<T>,
    /// The accuracy defining the end of game
    accuracy: T,
    window_size: NonZeroUsize,

    deltas: VecDeque<T>,

    n: usize,
    previous_price: Option<T>,
    price: T,
    sum_delta: T,
}

impl<'a, T: ComplexField> Iter<'a, T> {
    #[must_use]
    pub(super) fn new(
        game: &'a ContinuousConvexConcaveGame<T>,
        accuracy: T,
        window_size: NonZeroUsize,
    ) -> Self {
        // TODO: caller invariant on game properties
        Self {
            game,
            accuracy,
            deltas: VecDeque::with_capacity(window_size.get()),
            window_size,
            n: 1,
            previous_price: None,
            price: T::zero(),
            sum_delta: T::zero(),
        }
    }

    #[must_use]
    pub const fn n(&self) -> usize {
        self.n
    }
}

impl Iter<'_, f64> {
    /// Creates game matrix for the current iteration.
    ///
    /// # Panics
    ///
    /// If the resulting matrix cannot be created due to it being too big.
    fn current_game(&self) -> Game<DMatrix<f64>> {
        let dimension = self.n + 1;
        // check that we don't overflow
        dimension
            .checked_mul(dimension)
            .expect("the resulting matrix is too big");

        let divisor = self.n as f64;
        let data = (0..dimension)
            .flat_map(|j| (0..dimension).map(move |i| (i, j)))
            .map(|(i, j)| self.game.compute(i as f64 / divisor, j as f64 / divisor))
            .collect();

        Game::new(DMatrix::from_vec_storage(VecStorage::new(
            Dyn(dimension),
            Dyn(dimension),
            data,
        )))
    }
}

// TODO: generify on value type
impl Iterator for Iter<'_, f64> {
    type Item = GameSolution<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        self.n = self
            .n
            .checked_add(1)
            .expect("too many iterations have happened");

        let span = span!(Level::DEBUG, "CoCoCo-method iteration", n = self.n);
        let _enter = span.enter();
        trace!(delta = self.sum_delta, "Checking conditions");

        if self.deltas.is_empty() || self.sum_delta > self.accuracy {
            debug!(h = self.price, "Performing iterative step");
            trace!("Performing iteration");

            let game = self.current_game();
            debug!("Current game: {game:.3}");

            let (row, lowest_price) = game.lowest_price();
            trace!(
                "Lowest price: {:.3?} -> [{row}]: {lowest_price:.3}",
                game.min_win_a().as_slice()
            );
            let (column, highest_price) = game.highest_price();
            trace!(
                "Highest price: {:.3?} -> [{column}]: {highest_price:.3}",
                game.max_loss_b().as_slice()
            );

            let divisor = self.n as f64;
            // TODO: am I sure that comparing floats is okay?
            //  It should be given that they have the same source,
            //  but it may give false negatives on the values in different cells.
            let (price, x, y) = if lowest_price == highest_price {
                let span = span!(Level::TRACE, "Lo==Hi", price = lowest_price);
                let _enter = span.enter();

                let x = row as f64 / divisor;
                let y = column as f64 / divisor;
                debug!(x, y, price = lowest_price, "Saddle point found");
                (lowest_price, x, y)
            } else {
                let span = span!(Level::TRACE, "Lo!=Hi");
                let _enter = span.enter();

                trace!("Performing Brown-Robinson iteration");
                let mut brown_robinson = BrownRobinson::new(game.0);
                for BrownRobinsonRow { epsilon, .. } in &mut brown_robinson {
                    if epsilon < self.accuracy {
                        break;
                    }
                }
                let price = brown_robinson.price_estimation();
                let (a_strategy, b_strategy) = brown_robinson.strategies_used();
                let x = a_strategy.imax() as f64 / divisor;
                let y = b_strategy.imax() as f64 / divisor;
                debug!(x, y, price, "Brown-Robinson method completed");
                (price, x, y)
            };
            self.price = price;

            if let Some(previous_price) = self.previous_price {
                if self.deltas.len() == self.window_size.get() {
                    self.sum_delta -= self.deltas.pop_front().expect("window_size is non-zero");
                }

                let delta = (self.price - previous_price).abs();
                self.deltas.push_back(delta);
                self.sum_delta += delta;
            }
            self.previous_price = Some(self.price);

            Some(GameSolution {
                x,
                y,
                h: self.price,
            })
        } else {
            None
        }
    }
}

impl FusedIterator for Iter<'_, f64> {}
