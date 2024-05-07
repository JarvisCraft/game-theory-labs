mod coalition;

use std::{num::NonZeroU8, ops::Add};

use coalition::Coalition;

pub struct CooperativeGame<T>(Box<[T]>);

impl<T> CooperativeGame<T> {
    pub fn new(characteristic_function: Vec<T>) -> Result<Self, Vec<T>> {
        if !characteristic_function.is_empty() && characteristic_function.len().is_power_of_two() {
            Ok(Self(characteristic_function.into_boxed_slice()))
        } else {
            Err(characteristic_function)
        }
    }

    pub fn player_count(&self) -> NonZeroU8 {
        // The cast is always lossless since even `u128::MAX.ilog2() == 127`
        NonZeroU8::new(self.0.len().ilog2() as u8)
            .expect("the game is validated to be non-zero on creation")
    }

    pub fn coalitions(&self) -> impl Iterator<Item = Coalition> + Clone {
        (0..self.0.len()).map(Coalition)
    }

    pub fn v(&self, coalition: Coalition) -> &T {
        &self.0[coalition.0]
    }

    pub fn try_v(&self, coalition: Coalition) -> Option<&T> {
        self.0.get(coalition.0)
    }

    pub fn v_i(&self) -> &T {
        self.0.last().expect("the vector is known to not be empty")
    }

    pub fn singular_coalitions(&self) -> impl Iterator<Item = Coalition> + '_ {
        (0..self.player_count().get()).map(|player| Coalition(self.player_mask(player) as usize))
    }

    fn player_mask(&self, player: u8) -> u8 {
        let player_count = self.player_count();
        let Some(offset) = player_count
            .get()
            .checked_sub(player)
            .and_then(NonZeroU8::new)
        else {
            panic!("player={player} exceeds player_count={player_count}");
        };

        0b1 << (offset.get() - 1)
    }
}

impl<T: PartialOrd + Add<Output = T> + Clone> CooperativeGame<T> {
    pub fn is_super_additive(&self) -> bool {
        use itertools::Itertools;

        self.coalitions()
            .cartesian_product(self.coalitions())
            .filter(|(s, t)| !s.overlaps(*t))
            .all(|(s, t)| {
                let left = self.v(s | t);
                let right = self.v(s).clone() + self.v(t).clone();
                left >= &right
            })
    }

    pub fn is_convex(&self) -> bool
    where
        T: core::fmt::Display,
    {
        use itertools::Itertools;

        self.coalitions()
            .cartesian_product(self.coalitions())
            .all(|(s, t)| {
                let left = self.v(s | t).clone() + self.v(s & t).clone();
                let right = self.v(s).clone() + self.v(t).clone();
                println!("{} => {}", s | t, left);
                println!("{},{} => {}", s, t, right);
                left >= right
            })
    }
}

impl CooperativeGame<u8> {
    pub fn x(&self) -> impl Iterator<Item = f64> + '_ {
        let n = self.player_count().get();
        let n_factorial: f64 = (1..=n as u64).product::<u64>() as f64;

        (0..n).map(move |player| {
            let player_mask = self.player_mask(player) as usize;
            let i = Coalition(player_mask);

            let product: u64 = self
                .x_i(player)
                .map(|s| {
                    factorial(s.n_members() - 1)
                        * factorial(n - s.n_members())
                        * (self.v(s) - self.v(s - i)) as u64
                })
                .sum();
            product as f64 / n_factorial
        })
    }

    fn x_i(&self, player: u8) -> impl Iterator<Item = Coalition> {
        let player_mask = self.player_mask(player) as usize;
        self.coalitions()
            .filter(move |coalition| coalition.0 & player_mask != 0)
    }
}

fn factorial(n: u8) -> u64 {
    (1..=n as u64).product()
}

#[cfg(test)]
mod tests {
    use crate::cooperative::CooperativeGame;

    #[test]
    fn test_player_mask() {
        let game = CooperativeGame::new(vec![1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        assert_eq!(game.player_mask(0), 0b100);
        assert_eq!(game.player_mask(1), 0b010);
        assert_eq!(game.player_mask(2), 0b001);
    }

    #[test]
    fn factorial() {
        assert_eq!(super::factorial(0), 1);
        assert_eq!(super::factorial(1), 1);
        assert_eq!(super::factorial(2), 2);
        assert_eq!(super::factorial(3), 6);
    }
}
