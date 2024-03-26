use clap::Parser;

use continuous_convex_concave_method::ContinuousConcaveGame;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("h_xx={0} is not negative")]
    NonNegativeHxx(f64),
    #[error("h_yy={0} is not positive")]
    NonPositiveHyy(f64),
}

fn main() -> Result<(), Error> {
    let Options { a, b, c, d, e } = Options::parse();
    let game = ContinuousConcaveGame::new([a, b, c, d, e]);

    let (h_xx, h_yy) = (game.h_xx(), game.h_yy());
    println!("h_xx = {h_xx:.3}; h_yy = {h_yy:.3}");
    if h_xx >= 0. {
        return Err(Error::NonNegativeHxx(h_xx));
    }
    if h_yy <= 0. {
        return Err(Error::NonPositiveHyy(h_yy));
    }

    let (x_formula, y_formula) = game.x_y_formulas();
    println!("{{ {x_formula}");
    println!("{{ {y_formula}");

    let ((x, y), solved) = game.solve_analytically();
    print!("H({x:.3}, {y:.3}) = {solved:.3}");

    Ok(())
}

/// Finds optimal game strategies of a convex-concave zero-sum game
/// using analytical and numeric methods.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(allow_negative_numbers = true)]
struct Options {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
}
