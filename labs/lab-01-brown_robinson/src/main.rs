use std::{fs::File, path::PathBuf};

use clap::Parser;
use prettytable::{format::consts::FORMAT_BOX_CHARS, row, table};

use brown_robinson_method::{BrownRobinson, BrownRobinsonRow};
use game_theory::zero_sum::DGame;

fn main() {
    let Options {
        game,
        accuracy,
        output_file,
    } = Options::parse();

    let mut game = BrownRobinson::new(game.0);

    println!("Игра: {}", game.game());

    let (min, max) = game.bounds();
    println!("Нижняя цена игры: {min}, верхняя цена игры: {max}");
    let Some((a, b)) = game.game().solve_analytically() else {
        eprintln!("Система не имеет решений");
        return;
    };
    let (a_strat, a) = a.as_slice().split_last().unwrap();
    let (b_strat, b) = b.as_slice().split_last().unwrap();
    println!("Смешанная стратегия A: {a:.3?}");
    println!("Смешанная стратегия B: {b:.3?}",);
    println!("Цена игры: {:.3}~{:.3}", a_strat, b_strat);

    let mut table = table!([
        "k",
        "A",
        "B",
        "Стратегия A",
        "Стратегия B",
        "ВЦИ",
        "НЦИ",
        "ε"
    ]);
    table.set_format(*FORMAT_BOX_CHARS);

    // Запускаем итеративный алгоритм
    for BrownRobinsonRow {
        iteration,
        a_strategy,
        b_strategy,
        a_score,
        b_score,
        high_price,
        low_price,
        epsilon,
    } in &mut game
    {
        table.add_row(row![
            iteration,
            format!("x{}", a_strategy + 1),
            format!("y{}", b_strategy + 1),
            format!("{:.3?}", a_score.as_slice()),
            format!("{:.3?}", b_score.as_slice()),
            format!("{high_price:.3}"),
            format!("{low_price:.3}"),
            format!("{epsilon:.3}"),
        ]);

        if epsilon < accuracy {
            break;
        }
    }
    println!("{table}");

    let (&max_low_price, &min_high_price) = game.min_max_prices();
    let k = game.k();
    println!(
        "ВЦИ_min = {min_high_price:.3}, НЦИ_max = {max_low_price:.3}, ε[{k}] = {:.3}",
        (max_low_price + min_high_price) / 2.,
    );

    let (a_strategy_used, b_strategy_used) = game.strategies_used();
    println!(
        "x[{k}] = {}, y[{k}] = {}",
        a_strategy_used.map(|v| format!("{v}/{k}={:.3}", v as f64 / k as f64)),
        b_strategy_used.map(|v| format!("{v}/{k}={:.3}", v as f64 / k as f64))
    );

    if let Some(output_file) = output_file {
        match File::create(output_file) {
            Ok(file) => match table.to_csv(file) {
                Ok(_) => {
                    println!("CSV file generated successfully");
                }
                Err(e) => {
                    eprintln!("Failed to write CSV to file: {e}");
                }
            },
            Err(e) => {
                eprintln!("Failed to open file: {e}");
            }
        }
    }
}

/// Command line options of the program
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    /// Input game
    #[arg(long, short)]
    game: DGame<f64>,

    /// The required accuracy for the Brown-Robinson method
    #[arg(long, short, default_value_t = 0.1)]
    accuracy: f64,

    /// Name of the output file to which the CSV will be written.
    #[arg(long, short)]
    output_file: Option<PathBuf>,
}
