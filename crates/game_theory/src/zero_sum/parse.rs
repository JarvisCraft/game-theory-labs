use std::str::FromStr;

use nalgebra::{dmatrix, DMatrix, Dyn, VecStorage};
use peg::{error::ParseError, str::LineCol};

use super::{DGame, Game};
use crate::non_cooperative::{BiMatrixGame, Pair};

impl FromStr for DGame<f64> {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(game::dgame(s)?)
    }
}

impl FromStr for BiMatrixGame<f64> {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(game::bi_dgame(s)?)
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct FromStrError(#[from] ParseError<LineCol>);

peg::parser! {
    grammar game() for str {
        pub rule dgame<T: FromStr>() -> DGame<T>
            = "{" rows:((_ v:row() _ { v }) ** ";") _ ";"? _ "}"
        {?
            Ok(Game(dmatrix_from_rows(rows)?))
        }

        pub rule bi_dgame<T: FromStr>() -> BiMatrixGame<T>
            = "{" rows:((_ v:bi_row() _ { v }) ** ";") _ ";"? _ "}"
        {?
            Ok(BiMatrixGame::new(dmatrix_from_rows(rows)?))
        }

        rule _() = [' ' | '\t' | '\r' | '\n']*

        rule row<T: FromStr>() -> Vec<T>
            = "[" values:((_ v:float() _ { v }) ** ",") _ ","? _ "]"
        {
            values
        }

        rule bi_row<T: FromStr>() -> Vec<Pair<T>>
            = "[" values:((_ v:bi_float() _ { v }) ** ",") _ ","? _ "]"
        {
            values
        }

        rule bi_float<T: FromStr>() -> Pair<T> = "(" _ v1:float() _ "," _ v2:float() _ ")" {
            Pair(v1, v2)
        }

        rule float<T: FromStr>() -> T = num:$(sign()? finite_number()) {?
            T::from_str(num).or(Err("failed to parse float number"))
        }

        rule sign() -> bool = "+" { true } / "-" { false }

        rule digit() = ['0'..='9']

        rule exp() = "e" sign()? digit()+

        rule finite_number()
            = (digit()+) / (digit()+ "." digit()*) / (digit()* "." digit()+) exp()?
    }
}

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

    Ok(DMatrix::from_vec_storage(VecStorage::new(
        Dyn(row_count),
        Dyn(row_len),
        data,
    )))
}

#[cfg(test)]
mod tests {
    use nalgebra::dmatrix;

    use super::*;
    use crate::zero_sum::Game;

    #[test]
    fn multi_line_f64_matrix() {
        assert_eq!(
            game::dgame(
                "{
                    [1, 2, 3];
                    [4, 5, 6];
                }"
            ),
            Ok(Game(dmatrix![
                    1., 2., 3.;
                    4., 5., 6.;
            ])),
        );
    }

    #[test]
    fn single_line_f32_matrix() {
        assert_eq!(
            game::dgame("{[10,20] ; [30,40] ; [50,60] ; [70,80]}"),
            Ok(Game(dmatrix![
                    10f32, 20f32;
                    30f32, 40f32;
                    50f32, 60f32;
                    70f32, 80f32;
            ])),
        );
    }

    #[test]
    fn simple_bi_matrix() {
        assert_eq!(
            game::bi_dgame(
                "{
                    [(-5, -5), (0, -10)];
                    [(-10, 0), (-1, -1)];
                }"
            ),
            Ok(BiMatrixGame::new(dmatrix![
                Pair(-5., -5.), Pair(0., -10.);
                Pair(-10., 0.), Pair(-1., -1.);
            ])),
        );
    }
}
