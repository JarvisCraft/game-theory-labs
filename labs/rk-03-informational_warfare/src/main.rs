use std::{num::NonZeroU64, ops::DivAssign};

use clap::Parser;
use game_theory::generate::{random_matrix, random_vector};
use nalgebra::{DMatrix, DVector};
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
        a,
        b,
        c,
        d,
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

    let x = random_x(&mut random, dimensions, x_min, x_max);
    info!("x(0) = {:.03}", x.transpose());
    let (iteration, result_x) = simulate(&a, x.clone(), epsilon);
    info!("x({iteration}) = {:.03}", result_x.transpose());
    info!("A^{iteration} = {:.03}", a.pow(iteration as u32));

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

    println!(
        "Agents of Player 1: {:?}",
        agents_of_1.iter().map(|i| i + 1).collect::<Vec<_>>()
    );
    println!(
        "Agents of Player 2: {:?}",
        agents_of_2.iter().map(|i| i + 1).collect::<Vec<_>>()
    );

    let u = random.gen_range(agent_min..=agent_max.get()) as f64;
    let v = -(random.gen_range(agent_min..=agent_max.get()) as f64);
    let mut x_affected = x.clone();
    for &idx in &agents_of_1 {
        x_affected[idx] = u;
    }
    for &idx in &agents_of_2 {
        x_affected[idx] = v;
    }

    info!("x(0) = {:.03}", x_affected.transpose());
    let (iteration, result_x) = simulate(&a, x_affected, epsilon);
    info!("x({iteration}) = {:.03}", result_x.transpose());
    let a_final = a.pow(iteration as u32);
    info!("A^{iteration} = {:.03}", a_final);

    let r_f: f64 = a_final
        .row(0)
        .iter()
        .enumerate()
        .filter(|(index, _)| agents_of_1.contains(index))
        .map(|(_, &value)| value)
        .sum();
    let r_s: f64 = a_final
        .row(1)
        .iter()
        .enumerate()
        .filter(|(index, _)| agents_of_2.contains(index))
        .map(|(_, &value)| value)
        .sum();

    info!("Result for player 1: {r_f:.06}, Result for player 2: {r_s:.06}");

    // X = u * result_1 + v * result_2 + (x_affected?)
    info!("\\Phi_f(u, v) = a * X - b * X ** 2 - g_f * u ** 2 / 2");
    info!("\\Phi_s(u, v) = c * X - d * X ** 2 - g_s * v ** 2 / 2");

    // let f =
}

fn random_x(random: impl Rng, n: usize, min: u64, max: NonZeroU64) -> DVector<f64> {
    assert!(min < max.get());
    random_vector(random, n, min..=max.get(), |value| value as f64)
}

fn simulate(a: &DMatrix<f64>, mut x: DVector<f64>, epsilon: f64) -> (usize, DVector<f64>) {
    let mut iteration = 0;
    while x.max() - x.min() > epsilon {
        iteration += 1;
        x = a * &x;
        debug!("x({iteration}) = {}", x.transpose());
    }
    (iteration, x)
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

    #[arg(short, default_value_t = 1)]
    a: u32,

    #[arg(short, default_value_t = 1)]
    b: u32,

    #[arg(short, default_value_t = 1)]
    c: u32,

    #[arg(short, default_value_t = 1)]
    d: u32,

    #[arg(long, default_value_t = 1)]
    g_f: u32,

    #[arg(long, default_value_t = 1)]
    g_s: u32,
}
