use std::{
    fmt::{self, Display, Formatter},
    num::NonZeroUsize,
    write,
};

use formula::{XFormula, YFormula};
use game_theory::ext::ComplexFieldExt;
use iter::Iter;
use nalgebra::ComplexField;

mod formula;
mod iter;

/// A zero-sum game in a form:
///
/// ```latex
/// H(x, y) = ax^2 + by^2 + cxy + dx + ey
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContinuousConvexConcaveGame<T> {
    coefficients: [T; 5],
}

impl<T> ContinuousConvexConcaveGame<T> {
    #[must_use]
    pub fn new(coefficients: [T; 5]) -> Self {
        Self { coefficients }
    }
}

impl<T: ComplexField> ContinuousConvexConcaveGame<T> {
    /// Computes the value of the kernel function for the given parameters `x` and `y`.
    #[must_use]
    pub fn compute(&self, x: T, y: T) -> T {
        let Self {
            coefficients: [a, b, c, d, e],
        } = self;

        a.clone() * x.clone() * x.clone()
            + b.clone() * y.clone() * y.clone()
            + c.clone() * x.clone() * y.clone()
            + d.clone() * x
            + e.clone() * y
    }

    /// Computes the second-order partial derivative by `x`.
    #[must_use]
    pub fn h_xx(&self) -> T {
        let Self {
            coefficients: [a, _, _, _, _],
        } = self;

        a.clone() + a.clone()
    }

    /// Computes the second-order partial derivative by `y`.
    #[must_use]
    pub fn h_yy(&self) -> T {
        let Self {
            coefficients: [_, b, _, _, _],
        } = self;

        b.clone() + b.clone()
    }

    /// Computes the partial derivative by `x`.
    #[must_use]
    pub fn h_x(&self, x: T, y: T) -> T {
        let Self {
            coefficients: [a, _, c, d, _],
        } = self;

        let ax = a.clone() * x;
        ax.clone() + ax + c.clone() * y + d.clone()
    }

    /// Computes the partial derivative by `y`.
    #[must_use]
    pub fn h_y(&self, x: T, y: T) -> T {
        let Self {
            coefficients: [_, b, c, _, e],
        } = self;

        let by = b.clone() * y;
        by.clone() + by + c.clone() * x + e.clone()
    }

    /// Creates the separate formulas for `x` and `y` defined via each other.
    #[must_use]
    pub fn x_y_formulas(&self) -> (XFormula<T>, YFormula<T>) {
        let Self { coefficients } = self;
        let [a, b, c, d, e] = coefficients.clone();

        (XFormula { a, c: c.clone(), d }, YFormula { b, c, e })
    }

    /// Solves this formula producing the values of `x` and `y`
    /// and the corresponding `H(x,y)`.
    #[must_use]
    pub fn solve_analytically(&self) -> GameSolution<T> {
        let Self {
            coefficients: [a, b, c, d, e],
        } = self;
        // == The initial system is:
        // { x = y * (-c/(2a)) - d/(2a)
        // { y = x * (-c/(2b)) - e/(2b)
        //
        // == So let's define `x` via `y`:
        // x = [x * (-c/(2b)) - e/(2b)] * (-c/(2a)) - d/(2a)
        //   = x * c^2/(4ab) + ce/(4ab) - d/(2a)
        //
        // == Now move `x` to the left
        // x * [1 - c^2/(4ab)] = ce/(4ab) - d/(2a) = [ce/(2b) - d]/(2a)
        //
        // == Thus
        // x = [ce/(4ab) - d/(2a)] / [1 - c^2/(4ab)]
        //   = (ce - 2db) / (4ab - c^2)
        // of which `2b` part is re-usable

        let b_mul_2 = b.clone() * T::two();
        let x = (c.clone() * e.clone() - b_mul_2.clone() * d.clone())
            / (T::two() * a.clone() * b_mul_2.clone() - c.clone() * c.clone());
        let y = (-c.clone() * x.clone() - e.clone()) / b_mul_2;
        let h = self.compute(x.clone(), y.clone());

        GameSolution { x, y, h }
    }

    #[must_use]
    pub fn iter(&self, accuracy: T, window_size: NonZeroUsize) -> Iter<T> {
        Iter::new(self, accuracy, window_size)
    }
}

impl<T: Display> Display for ContinuousConvexConcaveGame<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self {
            coefficients: [a, b, c, d, e],
        } = self;
        write!(f, "H(x, y) = {a}x^2 + {b}y^2 + {c}xy + {d}x + {e}y")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameSolution<T> {
    pub x: T,
    pub y: T,
    pub h: T,
}
