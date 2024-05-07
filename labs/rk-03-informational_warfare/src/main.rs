use std::{num::NonZeroU64, ops::DivAssign};

use clap::Parser;
use game_theory::generate::random_matrix;
use nalgebra::DMatrix;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::{debug, error, info};

fn main() {
    let Options {
        dimensions,
        x_min,
        x_max,
        agent_min,
        agent_max,
        player_1_agents,
        player_2_agents,
        epsilon,
        seed,
        a: param_a,
        b: param_b,
        c: param_c,
        d: param_d,
        g_f,
        g_s,
    } = Options::parse();

    tracing_subscriber::fmt::init();

    if dimensions < 2 {
        error!("dimensions={dimensions} should be at least 2");
        return;
    }
    if x_min >= x_max.get() {
        error!("Minimal X value = {x_min} should be smaller than maximal X value = {x_max}");
        return;
    }

    if agent_min >= agent_max.get() {
        error!(
            "Minimal agent value = {agent_min} should be smaller than maximal agent value = {agent_max}"
        );
        return;
    }

    if player_1_agents + player_2_agents > dimensions {
        error!("The sum of player 1 agents = {player_1_agents} and player 2 agents = {player_2_agents} should not exceed {dimensions}");
        return;
    }

    let mut random = if let Some(seed) = seed {
        ChaCha20Rng::seed_from_u64(seed)
    } else {
        ChaCha20Rng::from_entropy()
    };

    let mut a = random_matrix(&mut random, dimensions, dimensions, 0. ..=1.);
    for mut row in a.row_iter_mut() {
        row.div_assign(row.sum());
    }
    info!("A = {a:.03}");
    let (iteration, a) = simulate(a, epsilon);
    info!("A^{iteration} = {a:.03}");

    let mut agents: Vec<_> = (0..dimensions).collect();
    let agents_of_1: Vec<_> = agents
        .choose_multiple(&mut random, player_1_agents)
        .copied()
        .collect();

    agents.retain(|agent| !agents_of_1.contains(agent));
    let agents_of_2: Vec<_> = agents
        .choose_multiple(&mut random, player_2_agents)
        .copied()
        .collect();

    info!(
        "Agents of Player 1: {:?}",
        agents_of_1.iter().map(|i| i + 1).collect::<Vec<_>>()
    );
    info!(
        "Agents of Player 2: {:?}",
        agents_of_2.iter().map(|i| i + 1).collect::<Vec<_>>()
    );

    let r_f: f64 = a
        .row(0)
        .iter()
        .enumerate()
        .filter(|(index, _)| agents_of_1.contains(index))
        .map(|(_, &value)| value)
        .sum();
    let r_s: f64 = a
        .row(1)
        .iter()
        .enumerate()
        .filter(|(index, _)| agents_of_2.contains(index))
        .map(|(_, &value)| value)
        .sum();
    info!("r_f = {r_f:.03}, r_s = {r_s:.03}");

    let u = u(param_a, param_b, param_c, param_d, g_f, g_s, r_f, r_s);
    let v = v(param_a, param_b, g_f, r_f, r_s, u);
    info!("u = {u:.03}, v = {v:.03}");

    let x = u * r_f + v * r_s;
    info!("Point of utopia: {x:.03}");

    let max_f = param_a / (2. * param_b);
    let max_s = param_c / (2. * param_d);
    info!("max_f = {max_f:.03}, max_s = {max_s:.03}");

    let d_f = (x - max_f).abs();
    let d_s = (x - max_s).abs();
    info!("d_f = {d_f:.03}, d_s = {d_s:.03}");

    if d_f < d_s  {
        info!("df < ds => player 1 wins");
    } else if d_f > d_s {
        info!("df > ds => player 2 wins");
    } else {
        info!("df == ds => draw");
    }
}

fn simulate(mut a: DMatrix<f64>, epsilon: f64) -> (usize, DMatrix<f64>) {
    let multiplier = a.clone();
    let mut iteration = 0;
    while a.column_iter().any(|row| row.max() - row.min() > epsilon) {
        iteration += 1;
        a *= &multiplier;
        debug!("A^{iteration} = {}", a);
    }
    (iteration, a)
}

fn u(a: f64, b: f64, c: f64, d: f64, g_f: f64, g_s: f64, r_f: f64, r_s: f64) -> f64 {
    (2. * (a * d - b * c) * r_f * r_s * r_s + a * g_s * r_f)
        / (2. * d * g_f * r_s * r_s + g_f * g_s + 2. * b * g_s * r_f * r_f)
}

fn v(a: f64, b: f64, g_f: f64, r_f: f64, r_s: f64, u: f64) -> f64 {
    (g_f * u + 2. * b * r_f * r_f * u - a * r_f) / (-2. * b * r_f * r_s)
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(allow_negative_numbers = true)]
struct Options {
    #[arg(long, short = 'n', default_value_t = 10)]
    dimensions: usize,

    #[arg(long, default_value_t = 1)]
    x_min: u64,

    #[arg(long, default_value_t = NonZeroU64::new(20).unwrap())]
    x_max: NonZeroU64,

    #[arg(long, default_value_t = 1)]
    agent_min: u64,

    #[arg(long, default_value_t = NonZeroU64::new(100).unwrap())]
    agent_max: NonZeroU64,

    #[arg(long, default_value_t = 2)]
    player_1_agents: usize,

    #[arg(long, default_value_t = 2)]
    player_2_agents: usize,

    #[arg(long, short, default_value_t = 1E-6)]
    epsilon: f64,

    /// Random generator seed
    #[arg(long)]
    seed: Option<u64>,

    #[arg(short, default_value_t = 4.)]
    a: f64,

    #[arg(short, default_value_t = 3.)]
    b: f64,

    #[arg(short, default_value_t = 2.)]
    c: f64,

    #[arg(short, default_value_t = 2.)]
    d: f64,

    #[arg(long, default_value_t = 3.)]
    g_f: f64,

    #[arg(long, default_value_t = 3.)]
    g_s: f64,
}
