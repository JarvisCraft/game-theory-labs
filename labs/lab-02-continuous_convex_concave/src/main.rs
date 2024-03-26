use std::num::NonZeroUsize;

use clap::Parser;
use continuous_convex_concave_method::{ContinuousConvexConcaveGame, GameSolution};
use tracing::info;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("h_xx={0} is not negative")]
    NonNegativeHxx(f64),
    #[error("h_yy={0} is not positive")]
    NonPositiveHyy(f64),
    #[error("there is no solution for the game")]
    NoSolution,
}

fn main() -> Result<(), Error> {
    let Options {
        a,
        b,
        c,
        d,
        e,
        accuracy,
        windows,
    } = Options::parse();

    tracing_subscriber::fmt::init();

    let game = ContinuousConvexConcaveGame::new([a, b, c, d, e]);

    let (h_xx, h_yy) = (game.h_xx(), game.h_yy());
    info!("h_xx = {h_xx:.3}; h_yy = {h_yy:.3}");
    if h_xx >= 0. {
        return Err(Error::NonNegativeHxx(h_xx));
    }
    if h_yy <= 0. {
        return Err(Error::NonPositiveHyy(h_yy));
    }

    let (x_formula, y_formula) = game.x_y_formulas();
    info!("{{ {x_formula}");
    info!("{{ {y_formula}");

    let GameSolution { x, y, h } = game.solve_analytically();
    info!("Analytically: H({x:.3}, {y:.3}) = {h:.3}");

    let GameSolution { x, y, h } = game
        .iter(accuracy, windows)
        .last()
        .ok_or(Error::NoSolution)?;
    info!("Iteratively: H({x:.3}, {y:.3}) = {h:.3}");

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

    /// The required accuracy for the iterative method
    #[arg(long, short, default_value_t = 0.1)]
    accuracy: f64,

    /// The size of the window for the iterative method
    #[arg(long, short, default_value_t = NonZeroUsize::new(10).unwrap())]
    windows: NonZeroUsize,
}
