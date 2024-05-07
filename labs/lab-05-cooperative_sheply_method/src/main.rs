use clap::Parser;
use game_theory::cooperative::CooperativeGame;
use tracing::{error, info, warn};

fn main() {
    let Options {
        characteristic_function,
    } = Options::parse();

    tracing_subscriber::fmt::init();

    let game = match CooperativeGame::new(characteristic_function) {
        Ok(game) => game,
        Err(characteristic_function) => {
            error!("Invalid number of coefficients in the characteristic function: {characteristic_function:?}");
            return;
        }
    };

    if game.is_super_additive() {
        info!("The game is super-additive");
    } else {
        warn!("The game is NOT super-additive")
    }

    if game.is_convex() {
        info!("The game is convex");
    } else {
        info!("The game is NOT convex")
    }

    let x: Vec<_> = game.x().collect();
    info!("Shapley value: {x:.03?}");

    let sum: f64 = x.iter().sum();
    let v_i = *game.v_i();
    if sum as u8 == v_i {
        info!("Group rationalism: V(I)={v_i} == sum={sum}")
    } else {
        warn!("NO Group rationalism: V(I)={v_i} != sum={sum}")
    }

    for (i, x, v) in x
        .iter()
        .zip(game.singular_coalitions())
        .enumerate()
        .map(|(index, (&x, i))| (index + 1, x, *game.v(i) as f64))
    {
        if x >= v {
            info!("Player {i} Individual rationalism: x_{i}={x:.03} >= v({{{i}}})={v:.03}");
        } else {
            warn!("Player {i} NO Individual rationalism: x_{i}={x:.03} < v({{{i}}})={v:.03}");
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    /// Ths characteristic function
    ///
    /// # Examples
    ///
    /// `0 1 1 3 1 3 3 4` for the example task.
    #[clap(default_values_t = vec![0, 1, 1, 2, 1, 2, 3, 6, 4, 7, 7, 10, 7, 10, 10, 12])]
    characteristic_function: Vec<u8>,
}
