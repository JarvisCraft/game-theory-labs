//! Extensions for the commonly used types.

use nalgebra::ComplexField;

/// Extension methods for [`ComplexField`].
pub trait ComplexFieldExt: ComplexField {
    /// Produces the value of `2` in this complex field.
    ///
    /// This is defined as:
    ///
    /// ```no_run
    /// two() == one() + one()
    /// ```
    fn two() -> Self {
        Self::one() + Self::one()
    }
}

impl<T: ComplexField> ComplexFieldExt for T {}
