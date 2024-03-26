use std::{collections::VecDeque, iter::FusedIterator, num::NonZeroUsize};

use brown_robinson_method::{BrownRobinson, BrownRobinsonRow};
use game_theory::zero_sum::Game;
use nalgebra::{ComplexField, DMatrix, Dyn, VecStorage};
use tracing::{info, span, trace, warn, Level};

use crate::ContinuousConvexConcaveGame;

pub struct Iter<T> {
    /// The iterated game
    game: ContinuousConvexConcaveGame<T>,
    /// The accuracy defining the end of game
    accuracy: T,
    window_size: NonZeroUsize,

    window: VecDeque<WindowElement<T>>,

    n: usize,
    previous_price: Option<T>,
    price: T,
    sum_delta: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State<T> {
    x: T,
    y: T,
    price: T,
}

#[derive(Debug, Clone)]
struct WindowElement<T> {
    delta: T,
    state: State<T>,
}

impl<T: ComplexField> Iter<T> {
    #[must_use]
    pub(super) fn new(
        game: ContinuousConvexConcaveGame<T>,
        accuracy: T,
        window_size: NonZeroUsize,
    ) -> Self {
        // TODO: caller invariant on game properties
        Self {
            game,
            accuracy,
            window: VecDeque::with_capacity(window_size.get()),
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

impl Iter<f64> {
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
impl Iterator for Iter<f64> {
    type Item = State<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        self.n = self
            .n
            .checked_add(1)
            .expect("too many iterations have happened");

        let span = span!(Level::TRACE, "CoCoCo-method iteration", n = self.n);
        let _enter = span.enter();
        trace!(delta = self.sum_delta, "Checking conditions");

        if self.window.is_empty() || self.sum_delta > self.accuracy {
            warn!("============================");
            warn!("n = {}", self.n);
            warn!("previous_price = {:?}", self.previous_price);
            warn!("price = {}", self.price);
            trace!("Performing iteration");

            let game = self.current_game();
            trace!("Current game: {game:.3}");

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
                trace!(x, y, price = lowest_price, "Saddle point found");
                (lowest_price, x, y)
            } else {
                let span = span!(Level::TRACE, "Lo!=Hi", lowest_price, highest_price);
                let _enter = span.enter();

                trace!("Performing brown robinson iteration");
                let mut brown_robinson = BrownRobinson::new(game.0);
                for BrownRobinsonRow { epsilon, .. } in &mut brown_robinson {
                    if epsilon < self.accuracy {
                        break;
                    }
                }
                let price = brown_robinson.price_estimation();
                let (a_strategy, b_strategy) = brown_robinson.strategies_used();
                println!("a = {a_strategy}");
                println!("b = {b_strategy}");
                let x = dbg!(a_strategy.imax()) as f64 / divisor;
                let y = dbg!(b_strategy.imax()) as f64 / divisor;
                trace!(x, y, price, "Brown-Robinson method completed");
                (price, x, y)
            };
            self.price = price;

            if let Some(previous_price) = self.previous_price {
                if self.window.len() == self.window_size.get() {
                    let WindowElement { delta, .. } =
                        self.window.pop_front().expect("window_size is non-zero");
                    self.sum_delta -= delta;
                }

                let delta = (self.price - previous_price).abs();
                self.window.push_back(WindowElement {
                    delta,
                    state: State {
                        x,
                        y,
                        price: self.price,
                    },
                });
                self.sum_delta += delta;
            }
            self.previous_price = Some(self.price);

            Some(State {
                x,
                y,
                price: self.price,
            })
        } else {
            None
        }
    }
}

impl FusedIterator for Iter<f64> {}
