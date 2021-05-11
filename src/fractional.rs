use crate::normalize_votes;
use std::collections::HashMap;

/// Real Quadratic Voting needs a market that people can exchange currencies
/// based on their willingness to vote.
/// The calculation above assumes that the vote exchange is conducted elsewhere
/// and the integrity of the the values of each vote is assured outside this module.
/// If this is not the case, and we want equal voting power among the partipipants,
/// `normalize` can be applied to balance votes.
pub struct FractionalVoting<'a> {
    voters: Vec<HashMap<&'a str, f64>>,
    normalize: bool,
    quadratic: bool,
}

impl<'a> FractionalVoting<'a> {
    pub fn new(voters: Vec<HashMap<&'a str, f64>>) -> Self {
        Self {
            voters,
            normalize: false,
            quadratic: false,
        }
    }

    // order matters

    pub fn normalize(&mut self, normalize: bool) {
        self.normalize = normalize;
    }

    pub fn quadratic(&mut self, quadratic: bool) {
        self.quadratic = quadratic;
    }

    pub fn calculate(&self) -> HashMap<&'a str, f64> {
        let mut result: HashMap<&'a str, f64> = HashMap::new();

        for voter in self.voters.iter() {
            let mut votes = voter.to_owned();

            if self.normalize {
                normalize_votes(&mut votes)
            }

            if self.quadratic {
                square_root_votes(&mut votes)
            }

            for (to, credit) in votes {
                let count = result.entry(to).or_insert(0f64);
                *count += credit;
            }
        }
        result
    }
}

pub fn square_root_votes<'a>(votes: &mut HashMap<&'a str, f64>) {
    for (_to, v) in votes.iter_mut() {
        *v = v.sqrt();
    }
}

#[cfg(test)]
mod fractional_test {

    use super::*;

    #[test]
    fn simple_quadratic() {
        let a: HashMap<&str, f64> = [("dog", 1f64), ("cat", 1f64), ("bat", 4f64)]
            .iter()
            .cloned()
            .collect();
        let b: HashMap<&str, f64> = [("dog", 16.0f64)].iter().cloned().collect();
        let c: HashMap<&str, f64> = [("cat", 4f64), ("bat", 9f64)].iter().cloned().collect();

        let votes = vec![a, b, c];

        let mut quad = FractionalVoting::new(votes);
        quad.quadratic(true);

        let result = quad.calculate();

        assert_eq!(result.get("dog"), Some(&5f64));
        assert_eq!(result.get("bat"), Some(&5f64));
    }

    #[test]
    fn simple_normalize() {
        let a: HashMap<&str, f64> = [("dog", 1f64), ("cat", 1f64), ("bat", 4f64)]
            .iter()
            .cloned()
            .collect();
        let b: HashMap<&str, f64> = [("dog", 16.0f64)].iter().cloned().collect();
        let c: HashMap<&str, f64> = [("cat", 4f64), ("bat", 9f64)].iter().cloned().collect();

        let votes = vec![a, b, c];

        let mut quad = FractionalVoting::new(votes);

        quad.quadratic(true);
        quad.normalize(true);

        let result = quad.calculate();

        eprint!("{:#?}", result);

        let dog = result.get("dog").unwrap();
        let bat = result.get("bat").unwrap();

        assert!(bat > dog); // bat wins
    }
}
