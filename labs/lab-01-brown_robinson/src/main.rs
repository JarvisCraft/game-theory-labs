use std::{
    fmt::{Display, Formatter},
    fs::File,
};

use brown_robinson_method::{solve_linear_equations, BrownRobinson, BrownRobinsonRow};
use prettytable::{format::consts::FORMAT_BOX_CHARS, row, table};

type Value = f64;

/// Тип для удобства, автоматически выполняющий округление значение при выводе.
#[derive(Debug)]
struct F(Value);

impl Display for F {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self(value) = self;
        write!(f, "{value:.3}")
    }
}

// Функция для удобства замены типа значений
fn v(value: u64) -> Value {
    value as Value
}

fn main() {
    // Условия задачи
    const ACCURACY: f64 = 0.1;
    #[cfg(not(feature = "example"))]
    let mut game = BrownRobinson::new([
        [v(12), v(9), v(18)],
        [v(15), v(22), v(5)],
        [v(16), v(3), v(12)],
    ]);

    #[cfg(feature = "example")]
    // The original game to ensure algorithm correctness:
    let mut game = BrownRobinson::new([[v(2), v(1), v(3)], [v(3), v(0), v(1)], [v(1), v(2), v(1)]]);

    println!("Игра: {}", game.game_matrix());

    let (min, max) = game.bounds();
    println!("Нижняя цена игры: {min}, верхняя цена игры: {max}");
    let a = solve_linear_equations(game.game_matrix().transpose());
    println!(
        "Смешанная стратегия A: ({:.3}, {:.3}, {:.3})",
        a[0], a[1], a[2]
    );
    let b = solve_linear_equations(*game.game_matrix());
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
            a_score[0],
            a_score[1],
            a_score[2],
            b_score[0],
            b_score[1],
            b_score[2],
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

    match File::create("output.csv") {
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
