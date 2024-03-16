//! Implementation of the iteration over Brown-Robinson method steps.

use std::iter::FusedIterator;

use nalgebra::Matrix1x3;
use ordered_float::NotNan;
use rand::prelude::SliceRandom;
use rand::thread_rng;

use super::{BrownRobinson, BrownRobinsonRow, Value};

impl BrownRobinson<Value> {
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
        (
            *a_indices.choose(&mut thread_rng()).unwrap(),
            *b_indices.choose(&mut thread_rng()).unwrap(),
        )
    }
}

impl Iterator for BrownRobinson<Value> {
    type Item = BrownRobinsonRow<Value>;

    /// Осуществляет шаг алгоритма Брауна-Робинсон.
    fn next(&mut self) -> Option<Self::Item> {
        self.k += 1;
        let (high_price, low_price) = if self.k == 1 {
            (self.high_price(), self.low_price())
        } else {
            let (a_strategy, b_strategy) = self.next_strategies();
            self.a_strategy = a_strategy;
            self.a_strategy_used[a_strategy] += 1;
            self.b_strategy = b_strategy;
            self.b_strategy_used[b_strategy] += 1;
            self.a_scores += Matrix1x3::from(self.game_matrix.row(b_strategy));
            self.b_scores += Matrix1x3::from(self.game_matrix.column(a_strategy).transpose());

            let high_price = self.high_price() / self.k as Value;
            let low_price = self.low_price() / self.k as Value;

            self.min_high_price = self.min_high_price.min(high_price);
            self.max_low_price = self.max_low_price.max(low_price);

            (high_price, low_price)
        };

        Some(BrownRobinsonRow {
            iteration: self.k,
            a_strategy: self.a_strategy,
            b_strategy: self.b_strategy,
            a_score: self.a_scores,
            b_score: self.b_scores,
            high_price,
            low_price,
            epsilon: self.min_high_price - self.max_low_price,
        })
    }
}

impl<T> FusedIterator for BrownRobinson<T> where Self: Iterator {}
