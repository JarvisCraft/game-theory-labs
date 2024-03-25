use std::str::FromStr;

use nalgebra::{dmatrix, DMatrix, Dyn, VecStorage};
use peg::{error::ParseError, str::LineCol};

use super::{DGame, Game};

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct FromStrError(#[from] ParseError<LineCol>);

/// Converts the rows into a dynamic matrix.
fn dmatrix_from_rows<T>(rows: Vec<Vec<T>>) -> Result<DMatrix<T>, &'static str> {
    let Some(row_len) = rows.first().map(Vec::len) else {
        return Ok(dmatrix![]);
    };

    let row_count = rows.len();
    let mut data = Vec::with_capacity(
        row_len
            .checked_mul(row_count)
            .ok_or("there are too many values in the matrix")?,
    );

    // `VecStorage` uses column-major order, so we have to transpose the matrix

    let mut rows: Vec<_> = rows.into_iter().map(|row| row.into_iter()).collect();
    for _ in 0..row_len {
        for row in &mut rows {
            data.push(row.next().ok_or("row lengths don't match")?);
        }
    }

    Ok(DMatrix::from_vec_storage(VecStorage::new(Dyn(row_count), Dyn(row_len), data)))
}

peg::parser! {
    grammar game() for str {
        pub rule dynamic() -> DGame<f64> = "{" _ rows:(row() ** ";") _ "}" {?
            Ok(Game(dmatrix_from_rows(rows)?))
        }

        // TODO: allow spaces between values
        rule row() -> Vec<f64> = "[" _ values:(float() ** ",") _ "]" { values }

        rule _() = [' ' | '\t' | '\r' | '\n']*

        rule float() -> f64 = num:$(sign()? finite_number()) {?
            f64::from_str(num).or(Err("failed to parse float number"))
        }

        rule sign() -> bool = "+" { true } / "-" { false }

        rule digit() = ['0'..='9']

        rule exp() = "e" sign()? digit()+

        rule finite_number()
            = (digit()+) / (digit()+ "." digit()*) / (digit()* "." digit()+) exp()?
    }
}

impl FromStr for DGame<f64> {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(game::dynamic(s)?)
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::dmatrix;

    use crate::zero_sum::Game;

    use super::*;

    #[test]
    fn print_deserialization() {
        let g = game::dynamic("{[1,2,3];[4,5,6]}").unwrap();
        println!("{}", g);
        for row in g.0.row_iter() {
            println!("Row: {row}");
        }
        assert_eq!(
            game::dynamic("{[1,2,3];[4,5,6]}"),
            Ok(Game(dmatrix![
                    1., 2., 3.;
                    4., 5., 6.;
            ])),
        );
    }
}
