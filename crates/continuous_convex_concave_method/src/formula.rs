use std::{
    fmt,
    fmt::{Display, Formatter},
};

use nalgebra::ComplexField;

use game_theory::ext::ComplexFieldExt;

/// `x` defined via `y`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XFormula<T> {
    pub a: T,
    pub c: T,
    pub d: T,
}

impl<T: ComplexField> XFormula<T> {
    /// Computes the value of `x` by `y`.
    #[must_use]
    pub fn compute(&self, y: T) -> T {
        let Self { a, c, d } = self;

        (-c.clone() * y - d.clone()) / (T::two() * a.clone())
    }
}

impl<T: Display> Display for XFormula<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { a, c, d } = self;
        write!(f, "x = (-{c}y - {d}) / (2 * {a})")
    }
}

/// `y` defined via `x`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct YFormula<T> {
    pub b: T,
    pub c: T,
    pub e: T,
}

impl<T: ComplexField> YFormula<T> {
    /// Computes the value of `y` by `x`.
    #[must_use]
    pub fn compute(&self, x: T) -> T {
        let Self { b, c, e } = self;

        (-c.clone() * x - e.clone()) / (T::two() * b.clone())
    }
}

impl<T: Display> Display for YFormula<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { b, c, e } = self;
        write!(f, "y = (-{c}x - {e}) / (2 * {b})")
    }
}
