use std::collections::HashMap;

mod approval;
mod fptp;
pub mod fractional;
pub mod liquid_democracy;
mod rcv;

pub fn normalize_votes<'a>(votes: &mut HashMap<&'a str, f64>) {
    let sum = votes.iter().fold(0f64, |acc, b| acc + b.1);

    for (_, v) in votes.iter_mut() {
        *v /= sum
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Ordf64(f64);

impl Ordf64 {
    pub fn new(n: f64) -> Self {
        if n.is_nan() {
            panic!("Ordf64 cannot be NaN")
        };
        Self(n)
    }
}

impl Eq for Ordf64 {}

impl Ord for Ordf64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}
