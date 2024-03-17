use std::{fs::File, path::PathBuf};

use clap::Parser;
use nalgebra::{ArrayStorage, matrix, Matrix1x3};
use prettytable::{format::consts::FORMAT_BOX_CHARS, row, table};

use brown_robinson_method::{BrownRobinson, BrownRobinsonRow};

fn main() {
    let options = Options::parse();

    // Условия задачи
    const ACCURACY: f64 = 0.1;

    let mut game = BrownRobinson::new(matrix![
        8., 12., 10.;
        1., 6., 19.;
        17., 11., 11.;
    ]);

    println!("Игра: {}", game.game());

    let (min, max) = game.bounds();
    println!("Нижняя цена игры: {min}, верхняя цена игры: {max}");
    let Some((a, b)) = game.game().solve_analytically() else {
        eprintln!("Система не имеет решений");
        return;
    };
    println!(
        "Смешанная стратегия A: ({:.3}, {:.3}, {:.3})",
        a[0], a[1], a[2]
    );
    println!(
        "Смешанная стратегия B: ({:.3}, {:.3}, {:.3})",
        b[0], b[1], b[2]
    );
    println!("Цена игры: {:.3}~{:.3}", a[3], b[3]);

    let mut table = table!([
        "k", "A", "B", "A:x1", "A:x2", "A:x3", "B:y1", "B:y2", "B:y3", "ВЦИ", "НЦИ", "ε"
    ]);
    table.set_format(*FORMAT_BOX_CHARS);

    // Запускаем итеративный алгоритм
    for BrownRobinsonRow {
        iteration,
        a_strategy,
        b_strategy,
        a_score:
            Matrix1x3 {
                data: ArrayStorage([[a0], [a1], [a2]]),
                ..
            },
        b_score:
            Matrix1x3 {
                data: ArrayStorage([[b0], [b1], [b2]]),
                ..
            },
        high_price,
        low_price,
        epsilon,
    } in &mut game
    {
        table.add_row(row![
            iteration,
            format!("x{}", a_strategy + 1),
            format!("y{}", b_strategy + 1),
            a0,
            a1,
            a2,
            b0,
            b1,
            b2,
            format!("{high_price:.3}"),
            format!("{low_price:.3}"),
            format!("{epsilon:.3}"),
        ]);

        if epsilon < ACCURACY {
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
        "x[{k}] = {:?}, y[{k}] = {:?}",
        a_strategy_used.map(|v| format!("{v}/{k}={:.3}", v as f64 / k as f64)),
        b_strategy_used.map(|v| format!("{v}/{k}={:.3}", v as f64 / k as f64))
    );

    if let Some(output_file) = options.output_file {
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

/// Command line op
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    /// Name of the output file to which the CSV will be written.
    #[arg(long, short)]
    output_file: Option<PathBuf>,
}
