use crate::{fractional::square_root_votes, normalize_votes, Ordf64};
use ndarray::{concatenate, s, Array, Array2, Axis};
use std::collections::{BTreeSet, HashMap};

pub type Voters<'a> = HashMap<&'a str, HashMap<&'a str, f64>>;

const ITERATION: u32 = 10_000;

#[derive(Debug)]
pub struct LiquidDemocracy<'a> {
    voters: HashMap<&'a str, HashMap<&'a str, f64>>,
    normalize: bool,
    quadratic: bool,
}

impl<'a> LiquidDemocracy<'a> {
    pub fn new(voters: Voters<'a>) -> Self {
        Self {
            voters,
            normalize: false,
            quadratic: false,
        }
    }

    pub fn normalize(&mut self, normalize: bool) {
        self.normalize = normalize
    }

    fn prepare_list(&self) -> (BTreeSet<&'a str>, BTreeSet<&'a str>) {
        let delegates: BTreeSet<&'a str> = self.voters.iter().map(|(n, _v)| n).cloned().collect();
        let set_vote_to: BTreeSet<&'a str> = self
            .voters
            .iter()
            .map(|(_, v)| v.iter().map(|(to, _v)| to))
            .flatten()
            .cloned()
            .collect();

        let policies: BTreeSet<&'a str> = set_vote_to.difference(&delegates).cloned().collect();

        (delegates, policies)
    }

    pub fn create_matrix(&self) -> ((BTreeSet<&'a str>, BTreeSet<&'a str>), Array2<f64>) {
        let (delegates, polices) = self.prepare_list();

        let mut d_to_p: Array2<f64> =
            Array::zeros((delegates.len() + polices.len(), delegates.len()));

        for (x, d) in delegates.iter().enumerate() {
            let mut votes = self.voters.get(d).unwrap().to_owned();

            if self.normalize {
                normalize_votes(&mut votes);
            }

            if self.quadratic {
                square_root_votes(&mut votes);
            }

            for (to, v) in votes {
                let y = match delegates.iter().position(|p| p == &to) {
                    Some(p) => p,
                    None => polices.iter().position(|p| p == &to).unwrap() + delegates.len(),
                };

                d_to_p[[y, x]] = v;
            }
        }

        let p_to_d: Array2<f64> = Array::zeros((polices.len(), delegates.len()));
        let p_to_p: Array2<f64> = Array::eye(polices.len());

        let left: Array2<f64> = concatenate![Axis(1), p_to_d, p_to_p];
        let matrix = concatenate![Axis(1), d_to_p, left.t()];

        ((delegates, polices), matrix)
    }

    pub fn calculate(&self) -> (HashMap<&'a str, f64>, HashMap<&'a str, f64>) {
        let ((delegates, polices), matrix) = self.create_matrix();

        let edge = matrix.shape()[0];
        let mut a = Array::eye(edge);
        let mut sum = Array::eye(edge);

        for _ in 0..ITERATION {
            a = a.dot(&matrix);
            sum += &a;
        }

        let a = a.slice(s![.., 0..delegates.len()]);
        let results = a.sum_axis(Axis(1)).slice(s![delegates.len()..]).to_vec();

        let poll_result: HashMap<&str, f64> = polices.iter().cloned().zip(results).collect();

        let sum = sum.slice(s![..delegates.len(), ..delegates.len()]);
        let sum_row = sum.sum_axis(Axis(1));
        let influence = (sum_row / sum.diag()).to_vec();

        let influence: HashMap<&str, f64> = delegates.iter().cloned().zip(influence).collect();

        (poll_result, influence)
    }
}

#[cfg(test)]
mod liquid_test {

    use super::*;

    fn breakfast<'a>() -> LiquidDemocracy<'a> {
        let minori = [
            ("yasushi", 0.1),
            ("ray", 0.1),
            ("rice", 0.1),
            ("bread", 0.7),
        ]
        .iter()
        .cloned()
        .collect();

        let yasushi = [("minori", 0.2), ("ray", 0.3), ("rice", 0.5)]
            .iter()
            .cloned()
            .collect();

        let ray = [("minori", 0.4), ("yasushi", 0.4), ("bread", 0.2)]
            .iter()
            .cloned()
            .collect();

        let voters: Voters = [("minori", minori), ("yasushi", yasushi), ("ray", ray)]
            .iter()
            .cloned()
            .collect();

        LiquidDemocracy::new(voters)
    }

    #[test]
    fn matrix_shape() {
        let liq = breakfast();
        let (_, matrix) = liq.create_matrix();

        assert_eq!(matrix.shape(), &[5, 5]);
    }

    #[test]
    fn simple() {
        let liq = breakfast();

        let (result, influence) = liq.calculate();

        let bread = result.get("bread").unwrap();
        let rice = result.get("rice").unwrap();

        assert!(rice < bread);

        let minori = influence.get("minori").unwrap();
        let yasushi = influence.get("yasushi").unwrap();

        assert!(minori > yasushi);
    }
}
