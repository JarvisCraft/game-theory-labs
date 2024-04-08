use std::fmt::{self, Display, Formatter};

use nalgebra::{DMatrix, Dim, Dyn, Matrix, RawStorage, VecStorage};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HighlightableCell<T> {
    Normal(T),
    Highlighted(T, char, char),
}

impl<T: Copy> HighlightableCell<T> {
    pub fn highlight(&mut self, left: char, right: char) {
        *self = match *self {
            HighlightableCell::Normal(value) => Self::Highlighted(value, left, right),
            HighlightableCell::Highlighted(value, _, _) => Self::Highlighted(value, left, right),
        }
    }
}

impl<T: Display> Display for HighlightableCell<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            HighlightableCell::Normal(value) => {
                write!(f, " {value} ")
            }
            HighlightableCell::Highlighted(value, left, right) => {
                write!(f, "{left}{value}{right}")
            }
        }
    }
}

pub trait WithHighlighting {
    type Highlighted;

    fn with_highlighting(self) -> Self::Highlighted;
}

impl<T: Clone, R: Dim, C: Dim, S: RawStorage<T, R, C>> WithHighlighting for Matrix<T, R, C, S> {
    type Highlighted = DMatrix<HighlightableCell<T>>;

    fn with_highlighting(self) -> Self::Highlighted {
        let (rows, columns) = (self.nrows(), self.ncols());
        DMatrix::from_vec_storage(VecStorage::new(
            Dyn(rows),
            Dyn(columns),
            self.iter()
                .cloned()
                .map(HighlightableCell::Normal)
                .collect(),
        ))
    }
}

pub trait Highlight {
    fn highlight(&mut self, row: usize, column: usize, left: char, right: char);
}

impl<T: Copy> Highlight for DMatrix<HighlightableCell<T>> {
    fn highlight(&mut self, row: usize, column: usize, left: char, right: char) {
        self[(row, column)].highlight(left, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foo() {
        use nalgebra::matrix;
        let mut x = matrix![
            1, 2, 3;
            4, 5, 6;
        ]
        .with_highlighting();
        x.highlight(1, 2, '(', ')');

        assert_eq!(
            x.to_string(),
            "
  ┌             ┐
  │  1   2   3  │
  │  4   5  (6) │
  └             ┘\n\n"
        );
    }
}
