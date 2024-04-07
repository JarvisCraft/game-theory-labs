use std::fmt::{self, Display, Formatter};

use nalgebra::{DMatrix, Dim, Dyn, Matrix, RawStorage, VecStorage};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BeautifulCell<T> {
    Normal(T),
    Highlighted(T),
}

impl<T: Display> Display for BeautifulCell<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BeautifulCell::Normal(value) => {
                write!(f, " {value} ")
            }
            BeautifulCell::Highlighted(value) => {
                write!(f, "({value})")
            }
        }
    }
}

pub trait Highlight {
    type Highlighted; // TODO: `Display`

    fn highlight(self, x: usize, y: usize) -> Self::Highlighted;
}

impl<T: Clone, R: Dim, C: Dim, S: RawStorage<T, R, C>> Highlight for Matrix<T, R, C, S> {
    type Highlighted = DMatrix<BeautifulCell<T> /*R, C, S*/>;

    fn highlight(self, row: usize, column: usize) -> Self::Highlighted {
        let (rows, columns) = (self.nrows(), self.ncols());
        let highlighted_index = row * columns + column;

        DMatrix::from_vec_storage(VecStorage::new(
            Dyn(rows),
            Dyn(columns),
            self.iter()
                .cloned()
                .enumerate()
                .map(|(index, item)| {
                    if index == highlighted_index {
                        BeautifulCell::Highlighted(item)
                    } else {
                        BeautifulCell::Normal(item)
                    }
                })
                .collect(),
        ))
    }
}

mod tests {
    use super::*;

    #[test]
    fn foo() {
        use nalgebra::matrix;
        assert_eq!(
            matrix![
                1, 2, 3;
                4, 5, 6;
            ]
            .highlight(1, 2)
            .to_string(),
            "
  ┌             ┐
  │  1   2   3  │
  │  4   5  (6) │
  └             ┘\n\n"
        );
    }
}
