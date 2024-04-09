use std::collections::HashSet;

use clap::Parser;
use game_theory::{
    highlight::{Highlight, WithHighlighting},
    non_cooperative::{BiMatrixGame, OptimalBiMatrixStrategy, Pair},
};
use nalgebra::dmatrix;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::{error, info, span, Level};

fn main() {
    let Options {
        random_game_dimension,
        seed,
        the_crossing_epsilon,
        game,
    } = Options::parse();
    tracing_subscriber::fmt::init();

    let random = if let Some(seed) = seed {
        ChaCha20Rng::seed_from_u64(seed)
    } else {
        ChaCha20Rng::from_entropy()
    };

    {
        print_delimiter()
        let _span = span!(Level::INFO, "Random matrix").entered();
        analyze_bi_matrix_game(BiMatrixGame::random(
            random,
            random_game_dimension,
            random_game_dimension,
            -50..50,
            f64::from,
        ));
    }

    {
        print_delimiter();
        let _span = span!(Level::INFO, "The Crossing").entered();
        analyze_bi_matrix_game(BiMatrixGame::new(dmatrix![
            Pair(1., 1.), Pair(1. - the_crossing_epsilon, 2.);
            Pair(2., 1. - the_crossing_epsilon), Pair(0., 0.);
        ]));
    }

    {
        print_delimiter();
        let _span = span!(Level::INFO, "The Family Conflict").entered();
        analyze_bi_matrix_game(BiMatrixGame::new(dmatrix![
            Pair(4., 1.), Pair(0., 0.);
            Pair(0., 0.), Pair(1., 4.);
        ]));
    }

    {
        print_delimiter();
        let _span = span!(Level::INFO, "Prisoner's dilemma").entered();
        analyze_bi_matrix_game(BiMatrixGame::new(dmatrix![
            Pair(-5., -5.), Pair(0., -10.);
            Pair(-10., 0.), Pair(-1., -1.);
        ]));
    }

    {
        print_delimiter();
        let _span = span!(Level::INFO, "The exact game").entered();
        analyze_bi_matrix_game(game.clone());

        if let Some(((v1, v2), (x, y))) = game.mixed_balanced_strategies() {
            info!("x = {x:.3}");
            info!("y = {y:.3}");
            info!("v1 = {v1:.3}, v2 = {v2:.3}");
        } else {
            error!("The game does not have a solution")
        }
    }
}

fn analyze_bi_matrix_game(game: BiMatrixGame<f64>) {
    info!("The original game: {game}");

    let mut nash = HashSet::new();
    {
        let nash_equilibriums = game.nash_equilibriums();
        let mut with_nash = game.0.clone().with_highlighting();
        let mut count = 0;
        for OptimalBiMatrixStrategy {
            wins: _,
            coordinate: (row, column),
        } in nash_equilibriums
        {
            with_nash.highlight(row, column, 'N', ' ');
            nash.insert((row, column));
            count += 1;
        }
        info!("{count} Nash equilibriums: {with_nash}");
    }

    let mut pareto = HashSet::new();
    {
        let pareto_efficients = game.pareto_efficients();
        let mut with_pareto = game.0.clone().with_highlighting();
        let mut count = 0;
        for OptimalBiMatrixStrategy {
            wins: _,
            coordinate: (row, column),
        } in pareto_efficients
        {
            with_pareto.highlight(row, column, 'P', ' ');
            pareto.insert((row, column));
            count += 1;
        }
        info!("{count} Pareto efficients: {with_pareto}");
    }

    {
        let mut with_intersection = game.0.with_highlighting();
        let mut has_intersections = false;
        let mut count = 0;
        for (row, column) in pareto.intersection(&nash).copied() {
            with_intersection.highlight(row, column, '*', '*');
            has_intersections = true;
            count += 1;
        }
        if has_intersections {
            info!("{count} intersections: {with_intersection}");
        } else {
            info!("No intersections");
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(allow_negative_numbers = true)]
struct Options {
    /// The dimension of the random game
    #[arg(long, short, default_value_t = 10)]
    random_game_dimension: usize,

    /// The required accuracy for the iterative method
    #[arg(long, short)]
    seed: Option<u64>,

    /// The value of epsilon in *The Crossing* game
    #[arg(long, short, default_value_t = 0.5)]
    the_crossing_epsilon: f64,

    /// The game to be solved
    #[arg(
        long,
        short,
        default_value = "{
            [(9, 8), (7, 4)];
            [(2, 1), (10, 3)];
        }"
    )]
    game: BiMatrixGame<f64>,
}

fn print_delimiter() {
    println!("{{================================================================}}")
}
