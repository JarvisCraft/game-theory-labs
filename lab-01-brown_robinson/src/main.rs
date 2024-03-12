use nalgebra::{Matrix1x3, Matrix3};
use ordered_float::NotNan;
use prettytable::{format::consts::FORMAT_BOX_CHARS, row, table};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::iter::FusedIterator;

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

const M: usize = 3;
const N: usize = 3;

pub struct BrownRobinsonRow {
    /// Номер текущей итерации
    iteration: usize,
    /// Текущая стратегия игрока A
    a_strategy: usize,
    /// Текущая стратегия игрока B
    b_strategy: usize,
    /// Накопленный выигрыш игрока A
    a_score: Matrix1x3<Value>,
    /// Накопленный выигрыш игрока B
    b_score: Matrix1x3<Value>,
    /// Верхняя цена игры
    high_price: Value,
    /// Нижняя цена игры
    low_price: Value,
    /// ε, разница между минимальной верхней и максиммальной нижней ценами игры
    epsilon: Value,
}

// Итератор по шагам метода
pub struct BrownRobinson {
    game_matrix: Matrix3<Value>,
    a_strategy: usize,
    b_strategy: usize,
    a_scores: Matrix1x3<Value>,
    b_scores: Matrix1x3<Value>,
    min_high_price: Value,
    max_low_price: Value,
    a_strategy_used: [usize; M],
    b_strategy_used: [usize; N],
    k: usize,
}

impl BrownRobinson {
    #[must_use]
    pub fn new(game_matrix: [[Value; M]; N]) -> Self {
        let game_matrix = Matrix3::from_data(nalgebra::ArrayStorage(game_matrix));
        let a_strategy = thread_rng().gen_range(0..M);
        let b_strategy = thread_rng().gen_range(0..N);

        let a_scores = game_matrix.row(a_strategy).clone_owned();
        let b_scores = game_matrix.column(b_strategy).transpose().clone_owned();
        let min_high_price = a_scores.max();
        let max_low_price = b_scores.min();

        let mut a_strategy_used = [0; M];
        a_strategy_used[a_strategy] = 1;
        let mut b_strategy_used = [0; N];
        b_strategy_used[b_strategy] = 1;

        Self {
            game_matrix,
            a_strategy,
            b_strategy,
            a_scores,
            b_scores,
            min_high_price,
            max_low_price,
            a_strategy_used,
            b_strategy_used,
            k: 0,
        }
    }

    #[must_use]
    pub fn bounds(&self) -> (Value, Value) {
        (self.max_low_price, self.min_high_price)
    }

    #[must_use]
    pub fn k(&self) -> usize {
        self.k
    }

    fn strategies_used(&self) -> ([usize; M], [usize; N]) {
        (self.a_strategy_used, self.b_strategy_used)
    }

    fn next_strategies(&self) -> (usize, usize) {
        let Self { a_scores, b_scores, .. } = self;

        let max_a = a_scores
            .iter()
            .copied()
            .max_by_key(|&value| NotNan::new(value).unwrap())
            .unwrap();
        let min_b = b_scores
            .iter()
            .copied()
            .min_by_key(|&value| NotNan::new(value).unwrap())
            .unwrap();

        let a_indices: Vec<_> = a_scores
            .iter()
            .enumerate()
            .filter(|(_, &value)| value == max_a)
            .map(|(index, _)| index)
            .collect();
        let b_indices: Vec<_> = b_scores
            .iter()
            .enumerate()
            .filter(|(_, &value)| value == min_b)
            .map(|(index, _)| index)
            .collect();
        (
            *a_indices.choose(&mut thread_rng()).unwrap(),
            *b_indices.choose(&mut thread_rng()).unwrap(),
        )
    }

    fn high_price(&self) -> Value {
        self.a_scores.max()
    }

    fn low_price(&self) -> Value {
        self.b_scores.min()
    }
}

impl Iterator for BrownRobinson {
    type Item = BrownRobinsonRow;

    /// Осуществляет шаг алгоритма Брауна-Робинсон.
    fn next(&mut self) -> Option<Self::Item> {
        self.k += 1;
        let (high_price, low_price) = if self.k == 1 {
            (self.high_price(), self.low_price())
        } else {
            let (a_strategy, b_strategy) = self.next_strategies();
            self.a_strategy = a_strategy;
            self.a_strategy_used[a_strategy] += 1;
            self.b_strategy = b_strategy;
            self.b_strategy_used[b_strategy] += 1;
            self.a_scores += Matrix1x3::from(self.game_matrix.row(b_strategy));
            self.b_scores += Matrix1x3::from(self.game_matrix.column(a_strategy).transpose());

            let high_price = self.high_price() / self.k as Value;
            let low_price = self.low_price() / self.k as Value;

            self.min_high_price = self.min_high_price.min(high_price);
            self.max_low_price = self.max_low_price.max(low_price);

            (high_price, low_price)
        };

        Some(BrownRobinsonRow {
            iteration: self.k,
            a_strategy: self.a_strategy,
            b_strategy: self.b_strategy,
            a_score: self.a_scores,
            b_score: self.b_scores,
            high_price,
            low_price,
            epsilon: self.min_high_price - self.max_low_price,
        })
    }
}

impl FusedIterator for BrownRobinson {}

fn main() {
    // Условия задачи
    const ACCURACY: f64 = 0.1;
    #[cfg(not(feature = "example"))]
    let mut game = BrownRobinson::new([
        [v(8), v(12), v(10)],
        [v(1), v(6), v(19)],
        [v(17), v(11), v(11)],
    ]);

    #[cfg(feature = "example")]
    // The original game to ensure algorithm correctness:
    let mut game = BrownRobinson::new([[v(2), v(1), v(3)], [v(3), v(0), v(1)], [v(1), v(2), v(1)]]);

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
            F(high_price),
            F(low_price),
            F(epsilon),
        ]);

        if epsilon < ACCURACY {
            break;
        }
    }
    println!("{table}");

    let (max_low_price, min_high_price) = game.bounds();
    let k = game.k();
    println!(
        "ВЦИ_min = {}, НЦИ_max = {}, ε[{k}] = {}",
        F(min_high_price), F(max_low_price), F((max_low_price + min_high_price) / 2.)
    );

    let (a_strategy_used, b_strategy_used) = game.strategies_used();
    println!(
        "x[{k}] = {:?}, y[{k}] = {:?}",
        a_strategy_used.map(|v| format!("{v}/{k}")),
        b_strategy_used.map(|v| format!("{v}/{k}"))
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
