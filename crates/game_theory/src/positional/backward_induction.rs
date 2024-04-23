use std::{
    collections::HashMap,
    fmt,
    fmt::{Debug, Display, Formatter},
    io::{self, Write},
    num::NonZeroU8,
};

use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    prelude::*,
};

#[derive(Debug)]
struct Loc {
    uid: usize,
    player: Player,
    strat: usize,
    parent: usize,
}

#[derive(Debug)]
struct Node<T> {
    loc: Loc,
    win: Option<Prize<T>>,
}

#[derive(Debug)]
struct Layer<T> {
    nodes: Vec<Node<T>>,
    size: usize,
}

#[derive(Debug)]
pub struct BackwardInductionGame<T> {
    layers: Vec<Layer<T>>,
}

impl<T> BackwardInductionGame<T> {
    pub fn reduce(&mut self, mut out: impl Write) -> io::Result<()>
    where
        T: Ord + Copy + Display,
    {
        writeln!(out, "# Iteration #0")?;
        writeln!(out)?;
        self.print_current(&mut out).unwrap();

        let mut iteration = 0;
        for layer in (1..self.layers.len()).rev() {
            iteration += 1;
            writeln!(out, "# Iteration #{iteration}")?;
            writeln!(out)?;

            let mut wins = HashMap::<usize, Vec<Prize<T>>>::new();
            for node in &self.layers[layer].nodes {
                wins.entry(node.loc.parent)
                    .or_default()
                    .push(node.win.clone().unwrap());
            }
            for (parent_idx, prizes) in wins {
                let parent = &mut self.layers[layer - 1].nodes[parent_idx];
                let parent_player = parent.loc.player.0;
                parent.win = Some(
                    prizes
                        .into_iter()
                        .max_by_key(|prize| prize.0[parent_player])
                        .unwrap(),
                )
            }

            self.print_current(&mut out)?;
        }

        Ok(())
    }

    pub fn random(
        mut generator: impl Rng,
        depth: NonZeroU8,
        players: &[NonZeroU8],
        range: impl SampleRange<T> + Clone,
    ) -> Option<Self>
    where
        T: SampleUniform,
    {
        if players.is_empty() {
            return None;
        }

        let depth = depth.get() as usize;
        let mut layers = Vec::with_capacity(depth);

        let mut uid = 0;
        let mut layer_size = 1usize;
        layers.push(Layer {
            nodes: vec![Node {
                loc: Loc {
                    uid,
                    player: Player(0),
                    strat: 0,
                    parent: 0,
                },
                win: None,
            }],
            size: layer_size,
        });

        for layer in 0..depth {
            let src_player = layer % players.len();

            let player_paths = players[src_player].get() as usize;

            layer_size *= player_paths;
            let mut nodes = Vec::with_capacity(layer_size);

            let mut parent_index = 0;
            for at_layer_index in 0..layer_size {
                uid += 1;

                let strat = at_layer_index % player_paths;
                if strat == 0 {
                    parent_index += 1;
                }

                nodes.push(Node {
                    loc: Loc {
                        uid,
                        player: Player((src_player + 1) % players.len()),
                        strat: strat + 1,
                        parent: parent_index - 1,
                    },
                    win: None,
                });
            }
            layers.push(Layer {
                nodes,
                size: layer_size,
            });
        }

        for node in &mut layers.last_mut().unwrap().nodes {
            node.win = Some(Prize(
                players
                    .iter()
                    .map(|_| generator.gen_range(range.clone()))
                    .collect(),
            ));
        }

        Some(Self { layers })
    }

    pub fn print_current(&self, out: &mut impl Write) -> io::Result<()>
    where
        T: Ord + Copy + Display,
    {
        writeln!(out, "```mermaid")?;
        writeln!(out, "flowchart LR")?;

        writeln!(out, "    0(({}))", Player(0))?;
        let max_layer = self.layers.len() - 1;
        for layer in 1..self.layers.len() {
            let prev_layer = &self.layers[layer - 1];
            let cur_layer = &self.layers[layer];

            let mut prev_index = 0;
            let mut wins = vec![];

            for cur_index in 0..cur_layer.nodes.len() {
                let cur = &cur_layer.nodes[cur_index];
                if cur.loc.strat == 1 {
                    prev_index += 1;
                    Win::commit(&wins, out)?;
                    wins.clear();
                }

                let prev = &prev_layer.nodes[prev_index - 1];
                if layer == max_layer {
                    if let Some(prize) = &cur.win {
                        writeln!(out, "    {} --> {}[[{}]]", prev.loc.uid, cur.loc.uid, prize)?;
                    } else {
                        writeln!(out, "    {} ---> {}[[_]]", prev.loc.uid, cur.loc.uid)?;
                    }
                } else {
                    writeln!(
                        out,
                        "    {0} ---> {1}(({2}))",
                        prev.loc.uid, cur.loc.uid, cur.loc.player
                    )?;
                }
                if let Some(prize) = cur.win.clone() {
                    wins.push(Win {
                        from_uid: prev.loc.uid,
                        to_uid: cur.loc.uid,
                        player: prev.loc.player,
                        prize,
                    });
                }
            }
            Win::commit(&wins, out)?;
        }
        writeln!(out, "```")?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct Player(usize);

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use fmt::Write;

        let Self(player) = self;
        f.write_char((b'A' + *player as u8) as char)
    }
}

#[derive(Clone, Debug)]
struct Prize<T>(Vec<T>);

impl<T: Display> Display for Prize<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use fmt::Write;

        let Self(wins) = self;
        let mut not_first = false;
        for win in wins {
            if not_first {
                f.write_str(", ")?;
            } else {
                not_first = true;
            }

            write!(f, "{win}")?;
        }

        Ok(())
    }
}

struct Win<T> {
    player: Player,
    from_uid: usize,
    to_uid: usize,
    prize: Prize<T>,
}
impl<T: Ord + Copy + Display> Win<T> {
    fn commit(wins: &Vec<Self>, out: &mut impl Write) -> io::Result<()> {
        let Some(max_win) = wins
            .iter()
            .map(|Win { player, prize, .. }| prize.0[player.0])
            .max()
        else {
            return Ok(());
        };

        for Win {
            from_uid,
            to_uid,
            prize,
            ..
        } in wins
            .iter()
            .filter(|Win { player, prize, .. }| prize.0[player.0] == max_win)
        {
            writeln!(out, "    {} ===>|\"{}\"| {}", to_uid, prize, from_uid)?;
        }

        Ok(())
    }
}
