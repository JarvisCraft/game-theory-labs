use std::iter;
use std::num::{NonZeroU32, NonZeroU8};

use clap::Parser;
use ordered_float::NotNan;
use prettytable::format::consts::FORMAT_BOX_CHARS;
use prettytable::row;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("invalid bounds: min = {min}, max = {max}")]
    InvalidPriceRange { min: u32, max: NonZeroU32 }
}

fn main() -> Result<(), Error> {
    let Options { n, min_price, max_price, seed } = Options::parse();
    if min_price >= max_price.get() {
        return Err(Error::InvalidPriceRange { min: min_price, max: max_price})
    }
    let range = min_price..=max_price.get();

    let mut random = if let Some(seed) = seed {
        ChaCha20Rng::seed_from_u64(seed)
    } else {
        ChaCha20Rng::from_entropy()
    };

    let values: Vec<_> = iter::repeat_with(|| random.gen_range(range.clone())).take(n.get().into()).collect();
    println!("Values: {values:?}");

    let bets = bets(values.iter().copied());
    println!("Bets: {bets:.3?}");

    let (winning_index, winning_bet) = winner(bets.iter().copied()).expect("the size is non-negative");

    let mut table = prettytable::table!([
        FrBybic->"Номер игрока",
        FrBybic->"Ценность",
        FrBybic->"Ставка",
        FrBybic->"Выигрыш",
    ]);
    table.set_format(*FORMAT_BOX_CHARS);

    println!("Победитель: Игрок #{} со ставкой: {winning_bet:.3}", winning_index - 1);
    for (index, (value, bet)) in iter::zip(values, bets).enumerate() {
        if index == winning_index {
            table.add_row(row![
                index + 1,
                value,
                format!("{bet:.3}"),
                format!("{:.3}", value as f64 - bet),
            ]);
        } else {
            table.add_row(row![index + 1, value, format!("{bet:.3}"), 0]);
        }
    }
    println!("{table}");

    Ok(())
}

fn bets(values: impl ExactSizeIterator<Item = u32>) -> Vec<f64> {
    let n = values.len() as f64;
    let multiplier = (n - 1.) / n;
    values.map(|value| value as f64 * multiplier).collect()
}

fn winner(values: impl IntoIterator<Item = f64>) -> Option<(usize, f64)> {
    values.into_iter().enumerate().max_by_key(|(_, value)| NotNan::new(*value).unwrap())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(allow_negative_numbers = true)]
struct Options {
    /// The number of candidate buyers
    #[arg(short)]
    n: NonZeroU8,
    #[arg(long, default_value_t = 0)]
    min_price: u32,
    #[arg(long, default_value_t = NonZeroU32::new(20_000).unwrap())]
    max_price: NonZeroU32,
    #[arg(short, long)]
    seed: Option<u64>,
}
