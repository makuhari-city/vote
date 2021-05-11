use std::collections::HashMap;

struct FirstPastThePostVoting<'a> {
    votes: Vec<&'a str>,
}

impl<'a> FirstPastThePostVoting<'a> {
    pub fn new(votes: Vec<&'a str>) -> Self {
        Self { votes }
    }

    pub fn calculate(&self) -> Vec<&'a str> {
        let mut counts: HashMap<&str, u32> = HashMap::new();

        for vote in self.votes.iter() {
            match counts.get_mut(vote) {
                Some(c) => *c += 1,
                None => {
                    counts.insert(vote, 1);
                }
            }
        }

        let mut max_count = 0u32;
        let mut winner = Vec::new();
        for (to, count) in counts.iter() {
            if &max_count < count {
                winner.clear();
                winner.push(*to);
                max_count = *count;
            } else if &max_count == count {
                winner.push(*to);
            }
        }

        return winner;
    }
}

#[cfg(test)]
mod fptp_test {

    use super::*;

    #[test]
    fn simple() {
        let votes = vec!["dog", "dog", "cat"];
        let fptp = FirstPastThePostVoting::new(votes);
        assert_eq!(fptp.calculate(), vec!["dog"]);
    }

    #[test]
    fn tie() {
        let votes = vec!["dog", "cat"];
        let fptp = FirstPastThePostVoting::new(votes);
        let result = fptp.calculate();
        assert!(result.contains(&"dog"));
        assert!(result.contains(&"cat"));

    }
}
