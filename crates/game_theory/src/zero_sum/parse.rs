use std::str::FromStr;

use nalgebra::{dmatrix, DMatrix, Dyn, VecStorage};
use peg::{error::ParseError, str::LineCol};

use super::Game;

#[derive(thiserror::Error, Debug)]
#[error("invalid matrix expression")]
pub struct FromStrError(#[from] ParseError<LineCol>);

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

    for row in rows {
        if row.len() != row_len {
            return Err("row lengths don't match");
        }
        data.extend(row);
    }

    let matrix = DMatrix::from_vec_storage(VecStorage::new(Dyn(row_count), Dyn(row_len), data));
    Ok(matrix)
}

peg::parser! {
    grammar game() for str {
        pub rule dynamic() -> Game<DMatrix<f64>> = "{" _ rows:(row() ** ";") _ "}" {?
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

impl FromStr for Game<DMatrix<f64>> {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(game::dynamic(s)?)
    }
}
//
// // TODO: generics
// impl<const N: usize, T: FromStr> FromStr
//     for Game<Matrix<T, Const<N>, Const<N>, ArrayStorage<T, N, N>>>
// {
//     type Err = FromStrError;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let rows = s.splitn(N, ';');
//         // for row in rows {
//         //     let drow = DVector::from_data(vec![1, 2]);
//         // }
//         let dynamic = DMatrix::from_vec_storage(VecStorage::new(Dyn(N), Dyn(N), vec![1; N * N]));
//
//         Ok(Game::new(dynamic.try_into().unwrap()))
//     }
// }

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
