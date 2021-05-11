use std::collections::HashMap;

// we only need the 'true' votes
struct ApprovalVoting<'a> {
    voters: Vec<Vec<&'a str>>,
    ignore: Vec<&'a str>,
}

impl<'a> ApprovalVoting<'a> {
    pub fn new(voters: Vec<Vec<&'a str>>) -> Self {
        Self {
            voters,
            ignore: Vec::new(),
        }
    }

    pub fn ignore(&mut self, ignore: &'a str) {
        self.ignore.push(ignore);
    }

    pub fn calculate(&self) -> Option<Vec<&'a str>> {
        let mut counts: HashMap<&str, u32> = HashMap::new();

        for voter in self.voters.iter() {
            for vote in voter {
                if !self.ignore.contains(vote) {
                    match counts.get_mut(vote) {
                        Some(c) => *c += 1,
                        None => {
                            counts.insert(vote, 1);
                        }
                    }
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

        if winner.is_empty() {
            return None;
        }

        Some(winner)
    }
}

impl<'a> Iterator for ApprovalVoting<'a> {
    type Item = Vec<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.calculate();

        if let Some(ignore_next) = result.as_ref() {
            for r in ignore_next {
                self.ignore(*r)
            }
        }

        result
    }
}

#[cfg(test)]
mod approval_test {

    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn simple() {
        let a = vec!["dog"];
        let b = vec!["cat"];
        let c = vec!["dog"];

        let voters: Vec<Vec<&str>> = vec![a, b, c];
        let rcv = ApprovalVoting::new(voters);
        assert_eq!(rcv.calculate(), Some(vec!["dog"]))
    }

    #[test]
    fn recurse() {
        let a = vec!["dog"];
        let b = vec!["cat"];
        let c = vec!["bat", "dog"];
        let d = vec!["dog"];

        let voters: Vec<Vec<&str>> = vec![a, b, c, d];
        let rcv = ApprovalVoting::new(voters);
        assert_eq!(rcv.calculate(), Some(vec!["dog"]))
    }

    #[test]
    fn no_majority() {
        let a = vec!["dog", "bat"];
        let b = vec!["cat"];
        let c = vec!["bat"];

        let voters: Vec<Vec<&str>> = vec![a, b, c];
        let rcv = ApprovalVoting::new(voters);
        assert_eq!(rcv.calculate(), Some(vec!["bat"]))
    }

    #[test]
    fn second_choice_wins() {
        // TODO: is this legal?
        let a = vec!["rat", "dog"];
        let b = vec!["cat", "dog"];
        let c = vec!["bat"];

        let voters: Vec<Vec<&str>> = vec![a, b, c];
        let rcv = ApprovalVoting::new(voters);
        assert_eq!(rcv.calculate(), Some(vec!["dog"]))
    }

    #[test]
    fn ignore() {
        let a = vec!["cat", "dog"];
        let b = vec!["bat"];
        let c = vec!["dog"];

        let voters: Vec<Vec<&str>> = vec![a, b, c];
        let mut rcv = ApprovalVoting::new(voters);
        rcv.ignore("cat");
        assert_eq!(rcv.calculate(), Some(vec!["dog"]));
    }

    #[test]
    fn iterator() {
        let a = vec!["cat", "dog"];
        let b = vec!["cat"];
        let c = vec!["dog"];

        let voters: Vec<Vec<&str>> = vec![a, b, c];
        let rcv = ApprovalVoting::new(voters);

        let mut rcv_iter = rcv.into_iter();

        let set: BTreeSet<&str> = ["dog", "cat"].iter().cloned().collect();
        let result: BTreeSet<&str> = rcv_iter.next().unwrap().iter().cloned().collect();

        assert_eq!(set, result);
        assert_eq!(rcv_iter.next(), None);
    }
}
