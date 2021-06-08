use std::collections::HashMap;

#[derive(Debug)]
pub struct BordaCount<'a> {
    voters: Vec<Vec<&'a str>>,
}

impl<'a> BordaCount<'a> {
    pub fn new(voters: Vec<Vec<&'a str>>) -> Self {
        Self { voters }
    }

    pub fn calculate(&self) -> HashMap<&'a str, f64> {
        let mut result: HashMap<&'a str, f64> = HashMap::new();

        for votes in self.voters.iter() {
            for (i, vote) in votes.iter().enumerate() {
                let value = (votes.len() - i) as f64;
                match result.get_mut(vote) {
                    Some(v) => {
                        *v += value;
                    }
                    None => {
                        result.insert(vote, value);
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod borda_test {
    use super::*;

    fn dinner<'a>() -> BordaCount<'a> {
        let minori = vec!["beef steak", "kungpao chicken", "white pork stew"];
        let yasushi = vec!["kungpao chicken", "beef steak", "white pork stew"];
        let ray = vec!["white pork stew", "beef steak", "kungpao chicken"];
        let sola = vec!["white pork stew", "kungpao chicken", "beef steak"];

        let voters: Vec<Vec<&'a str>> = vec![minori, yasushi, ray, sola].iter().cloned().collect();

        BordaCount::new(voters)
    }

    #[test]
    fn simple() {
        let borda = dinner();
        let result = borda.calculate();
        let chicken = result.get("kungpao chicken").unwrap();
        let pork = result.get("white pork stew").unwrap();

        assert_eq!(pork, &8.0);
        assert_eq!(chicken, &8.0);
    }
}
