//! Implementation of the iteration over Brown-Robinson method steps.

use std::iter::FusedIterator;

use nalgebra::{allocator::Allocator, DefaultAllocator, Dim, Storage, U1};
use ordered_float::NotNan;
use rand::{prelude::SliceRandom, thread_rng};
use tracing::{instrument, span, trace, Level};

use super::{BrownRobinson, BrownRobinsonRow};

type T = super::Value;

impl<N: Dim, S: Storage<T, N, N>> BrownRobinson<T, N, S>
where
    DefaultAllocator: Allocator<usize, U1, N> + Allocator<T, U1, N>,
{
    #[instrument("Selecting strategies", skip_all)]
    fn next_strategies(&self) -> (usize, usize) {
        let Self {
            a_scores, b_scores, ..
        } = self;

        let max_a = a_scores
            .iter()
            .copied()
            .max_by_key(|&value| NotNan::new(value).unwrap())
            .unwrap();
        let min_b = b_scores
            .iter()
            .copied()
            .min_by_key(|&value| NotNan::new(value).unwrap())
            .unwrap();

        trace!(
            "A = {:.3?}, min_b = {:.3?}",
            a_scores.as_slice(),
            b_scores.as_slice()
        );
        trace!("max_a = {max_a:.3}, min_b = {min_b:.3}");

        let a_indices: Vec<_> = a_scores
            .iter()
            .enumerate()
            .filter(|(_, &value)| value == max_a)
            .map(|(index, _)| index)
            .collect();
        let b_indices: Vec<_> = b_scores
            .iter()
            .enumerate()
            .filter(|(_, &value)| value == min_b)
            .map(|(index, _)| index)
            .collect();
        let (a, b) = (
            *a_indices.choose(&mut thread_rng()).unwrap(),
            *b_indices.choose(&mut thread_rng()).unwrap(),
        );
        trace!("Selected strategies: [{a}][{b}]");
        (a, b)
    }
}

impl<N: Dim, S: Storage<T, N, N>> Iterator for BrownRobinson<T, N, S>
where
    DefaultAllocator: Allocator<usize, U1, N> + Allocator<T, U1, N>,
{
    type Item = BrownRobinsonRow<T, N>;

    /// Осуществляет шаг алгоритма Брауна-Робинсон.
    fn next(&mut self) -> Option<Self::Item> {
        self.k += 1;
        let span = span!(Level::TRACE, "Brown-Robinoson step", k = self.k);
        let _enter = span.enter();

        let (high_price, low_price) = if self.k == 1 {
            trace!("Performing initial (no-op) iteration");
            (self.high_price(), self.low_price())
        } else {
            let (a_strategy, b_strategy) = self.next_strategies();
            self.a_strategy = a_strategy;
            self.a_strategy_times_used[a_strategy] += 1;
            self.b_strategy = b_strategy;
            self.b_strategy_times_used[b_strategy] += 1;
            self.a_scores += self.game.0.column(b_strategy).transpose();
            self.b_scores += self.game.0.row(a_strategy);

            let high_price = self.high_price() / self.k as T;
            let low_price = self.low_price() / self.k as T;

            self.min_high_price = self.min_high_price.min(high_price);
            self.max_low_price = self.max_low_price.max(low_price);

            (high_price, low_price)
        };
        trace!("Produced prices: ({high_price:.3}; {low_price:.3})");

        Some(BrownRobinsonRow {
            iteration: self.k,
            a_strategy: self.a_strategy,
            b_strategy: self.b_strategy,
            a_score: self.a_scores.clone_owned(),
            b_score: self.b_scores.clone_owned(),
            high_price,
            low_price,
            epsilon: self.min_high_price - self.max_low_price,
        })
    }
}

impl<N: Dim, S: Storage<T, N, N>> FusedIterator for BrownRobinson<T, N, S> where
    DefaultAllocator: Allocator<usize, U1, N> + Allocator<T, U1, N>
{
}
