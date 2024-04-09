use std::fmt::{self, Display, Formatter};

use crate::non_cooperative::{BiMatrixGame, Pair};

#[derive(Debug, Clone)]
pub struct OptimalBiMatrixStrategy<'a, T> {
    pub wins: &'a Pair<T>,
    pub coordinate: (usize, usize),
}

impl<T: Display> Display for OptimalBiMatrixStrategy<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self {
            wins: Pair(win_a, win_b),
            coordinate: (row, column),
        } = self;
        write!(f, "{row}:{column} -> ({win_a}, {win_b})")
    }
}

impl<T> BiMatrixGame<T> {
    pub fn nash_equilibriums(&self) -> impl Iterator<Item = OptimalBiMatrixStrategy<'_, T>>
    where
        T: PartialOrd,
    {
        self.optimal_by(Self::is_nash_equilibrium)
    }

    pub fn pareto_efficients(&self) -> impl Iterator<Item = OptimalBiMatrixStrategy<'_, T>>
    where
        T: PartialOrd,
    {
        self.optimal_by(Self::is_pareto_efficient)
    }

    fn optimal_by(
        &self,
        filter: impl Fn(&Self, (usize, usize)) -> bool,
    ) -> impl Iterator<Item = OptimalBiMatrixStrategy<'_, T>> {
        let Self(game) = self;
        (0..game.nrows())
            .flat_map(|row| (0..game.ncols()).map(move |column| (row, column)))
            .filter_map(move |coordinate| {
                if filter(self, coordinate) {
                    Some(self.optimal_at(coordinate))
                } else {
                    None
                }
            })
    }

    fn is_nash_equilibrium(&self, (row, column): (usize, usize)) -> bool
    where
        T: PartialOrd,
    {
        let Self(game) = self;
        let Pair(win_a, win_b) = &game[(row, column)];

        if (0..game.nrows()).any(|other_row| game[(other_row, column)].0 > *win_a) {
            false
        } else {
            (0..game.ncols()).all(|other_column| game[(row, other_column)].1 <= *win_b)
        }
    }

    fn is_pareto_efficient(&self, (row, column): (usize, usize)) -> bool
    where
        T: PartialOrd,
    {
        let Self(game) = self;
        let Pair(win_a, win_b) = &game[(row, column)];

        (0..game.nrows())
            .flat_map(|row| (0..game.ncols()).map(move |column| &game[(row, column)]))
            .all(|Pair(other_win_a, other_win_b)| {
                (other_win_a < win_a || other_win_b < win_b)
                    || (other_win_a <= win_a && other_win_b <= win_b)
            })
    }

    fn optimal_at(&self, coordinate: (usize, usize)) -> OptimalBiMatrixStrategy<T> {
        OptimalBiMatrixStrategy {
            wins: &self.0[coordinate],
            coordinate,
        }
    }
}
