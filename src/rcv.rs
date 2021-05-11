use std::collections::{BTreeSet, HashMap};

struct RankChoiceVoting<'a> {
    voters: Vec<Vec<&'a str>>,
    ignore: Vec<&'a str>,
}

impl<'a> RankChoiceVoting<'a> {
    pub fn new(voters: Vec<Vec<&'a str>>) -> Self {
        Self {
            voters,
            ignore: Vec::new(),
        }
    }

    pub fn ignore(&mut self, vote: &'a str) {
        self.ignore.push(vote);
    }

    fn unique_votes(&self) -> BTreeSet<&'a str> {
        let mut set = BTreeSet::new();

        for voter in self.voters.iter() {
            for vote in voter {
                set.insert(*vote);
            }
        }

        set
    }

    pub fn calculate(&self) -> Option<Vec<&'a str>> {
        let mut eliminate = Vec::new();
        let unique = self.unique_votes();
        let mut counts: HashMap<&str, u32> = HashMap::new();

        loop {
            for voter in self.voters.iter() {
                for vote in voter {
                    if !eliminate.contains(vote) && !self.ignore.contains(vote) {
                        match counts.get_mut(vote) {
                            Some(c) => *c += 1,
                            None => {
                                counts.insert(vote, 1);
                            }
                        }
                        break;
                    }
                }
            }

            let mut winners = Vec::new();
            let majority = (self.voters.len() as u32) / 2;

            for (to, count) in counts.iter() {
                println!("{}: {}", to, count);
                if count > &majority {
                    winners.push(*to);
                }
            }

            if !winners.is_empty() {
                return Some(winners);
            }

            let mut min_count = u32::MAX;
            for (to, count) in counts.iter() {
                if count < &min_count {
                    min_count = *count;
                    eliminate.clear();
                    eliminate.push(to);
                } else if count == &min_count {
                    eliminate.push(to);
                }
            }

            if eliminate.len() == unique.len() {
                return None;
            }
        }
    }
}

impl<'a> Iterator for RankChoiceVoting<'a> {
    type Item = Vec<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.calculate();
        if let Some(winners) = &result {
            for w in winners {
                self.ignore(*w);
            }
        }
        result
    }
}

#[cfg(test)]
mod rcv_test {

    use super::*;

    #[test]
    fn simple() {
        let a = vec!["dog"];
        let b = vec!["cat"];
        let c = vec!["dog"];

        let voters: Vec<Vec<&str>> = vec![a, b, c];
        let rcv = RankChoiceVoting::new(voters);
        assert_eq!(rcv.calculate(), Some(vec!["dog"]))
    }

    #[test]
    fn recurse() {
        let a = vec!["dog"];
        let b = vec!["cat"];
        let c = vec!["bat", "dog"];
        let d = vec!["dog"];

        let voters: Vec<Vec<&str>> = vec![a, b, c, d];
        let rcv = RankChoiceVoting::new(voters);
        assert_eq!(rcv.calculate(), Some(vec!["dog"]))
    }

    #[test]
    fn no_majority() {
        let a = vec!["dog", "bat"];
        let b = vec!["cat"];
        let c = vec!["bat"];

        let voters: Vec<Vec<&str>> = vec![a, b, c];
        let rcv = RankChoiceVoting::new(voters);
        assert_eq!(rcv.calculate(), None)
    }

    #[test]
    fn second_choice_wins() {
        // TODO: is this legal?
        let a = vec!["rat", "dog"];
        let b = vec!["cat", "dog"];
        let c = vec!["bat"];

        let voters: Vec<Vec<&str>> = vec![a, b, c];
        let rcv = RankChoiceVoting::new(voters);
        assert_eq!(rcv.calculate(), Some(vec!["dog"]))
    }

    #[test]
    fn second_choice_no_majority() {
        let a = vec!["rat", "dog"];
        let b = vec!["cat"];
        let c = vec!["bat"];

        let voters: Vec<Vec<&str>> = vec![a, b, c];
        let rcv = RankChoiceVoting::new(voters);
        assert_eq!(rcv.calculate(), None)
    }

    #[test]
    fn ignore() {
        let a = vec!["cat", "dog"];
        let b = vec!["bat"];
        let c = vec!["dog"];

        let voters: Vec<Vec<&str>> = vec![a, b, c];
        let mut rcv = RankChoiceVoting::new(voters);
        rcv.ignore("cat");
        assert_eq!(rcv.calculate(), Some(vec!["dog"]))
    }

    #[test]
    fn double_round() {
        let a = vec!["cat", "dog"];
        let b = vec!["cat"];
        let c = vec!["dog"];

        let voters: Vec<Vec<&str>> = vec![a, b, c];
        let mut rcv = RankChoiceVoting::new(voters);
        let first = rcv.calculate();
        assert_eq!(first, Some(vec!["cat"]));
        rcv.ignore("cat");
        assert_eq!(rcv.calculate(), Some(vec!["dog"]));
    }

    #[test]
    fn iterator() {
        let a = vec!["cat", "dog"];
        let b = vec!["cat"];
        let c = vec!["dog"];

        let voters: Vec<Vec<&str>> = vec![a, b, c];
        let rcv = RankChoiceVoting::new(voters);

        let mut rcv_iter = rcv.into_iter();

        assert_eq!(rcv_iter.next(), Some(vec!["cat"]));
        assert_eq!(rcv_iter.next(), Some(vec!["dog"]));
    }
}
