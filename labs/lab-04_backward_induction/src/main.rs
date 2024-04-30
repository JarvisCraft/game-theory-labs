use std::{fs::File, num::NonZeroU8, path::PathBuf};

use clap::Parser;
use game_theory::positional::backward_induction::BackwardInductionGame;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::error;

fn main() {
    let Options {
        depth,
        min,
        max,
        strategies,
        seed,
        output,
    } = Options::parse();

    tracing_subscriber::fmt::init();

    if min >= max {
        error!("Min={min} should be smaller than Max={max}");
        return;
    }

    let random = if let Some(seed) = seed {
        ChaCha20Rng::seed_from_u64(seed)
    } else {
        ChaCha20Rng::from_entropy()
    };

    let Some(mut tree) = BackwardInductionGame::random(random, depth, &strategies, min..=max)
    else {
        error!("There are not strategies defined");
        return;
    };

    let out = match File::create(&output) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to create output file {output:?}: {e}");
            return;
        }
    };

    match tree.reduce(out) {
        Ok(()) => {}
        Err(e) => error!("Failed to reduce the tree: {e}"),
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(allow_negative_numbers = true)]
struct Options {
    /// The depth of the tree
    #[arg(long, short, default_value_t = NonZeroU8::new(6).unwrap())]
    depth: NonZeroU8,

    /// The minimal win (inclusive)
    #[arg(long, default_value_t = -20)]
    min: i32,

    /// The maximal win (inclusive)
    #[arg(long, default_value_t = 15)]
    max: i32,

    /// Ths strategies of the players
    #[arg(long, short)]
    strategies: Vec<NonZeroU8>,

    /// Random generator seed
    #[arg(long)]
    seed: Option<u64>,

    #[arg(long, short)]
    output: PathBuf,
}
