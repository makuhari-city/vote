use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// Votes shows how the users votes are handled.
/// The first Uuid must be one assigned to the `delegates` field,
/// yet the second one does not restrict it to `polices`
pub type Votes = BTreeMap<Uuid, BTreeMap<Uuid, f64>>;

/// Topic is a high level struct that has comprehensive info about the `topic`.
/// It has two purposes:
///   1. a data format that is ready to be used in the front end,
///   2. a format that is saved in a database
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Topic {
    id: Uuid,
    title: String,
    description: String,
    /// This is hash of the previous hash
    parent: Option<String>,
    delegates: BTreeMap<Uuid, String>,
    policies: BTreeMap<Uuid, String>,
    votes: Votes,
    results: BTreeMap<String, Value>,
}

impl Topic {
    pub fn new(title: &str, description: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: description.to_string(),
            parent: None,
            delegates: BTreeMap::new(),
            policies: BTreeMap::new(),
            votes: BTreeMap::new(),
            results: BTreeMap::new(),
        }
    }

    pub fn add_new_delegate(&mut self, nickname: &str) -> Option<Uuid> {
        if self.delegates.iter().any(|(_uid, name)| name == nickname) {
            None
        } else {
            let id = Uuid::new_v4();
            self.delegates.insert(id.to_owned(), nickname.to_string());
            Some(id)
        }
    }

    pub fn add_new_policy(&mut self, title: &str) -> Option<Uuid> {
        if self.policies.iter().any(|(_uid, t)| t == title) {
            None
        } else {
            let id = Uuid::new_v4();
            self.policies.insert(id.to_owned(), title.to_string());
            Some(id)
        }
    }

    pub fn cast_vote_to(&mut self, src: &Uuid, target: &Uuid, value: f64) {
        //TODO : check for illegal votes
        self.votes
            .get_mut(src)
            .and_then(|user_votes| user_votes.insert(target.to_owned(), value));
    }
}

/// VoteInfo is a redacted version of `Topic`.
/// The calculation modules will not need to know the full information about the topic.
/// This struct strictly defines only the necessary information for aggregating votes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoteInfo {
    /// notice the data format is different from the one in `Topic`
    delegates: BTreeSet<Uuid>,
    policies: BTreeSet<Uuid>,
    votes: Votes,
}

impl VoteInfo {
    /// Some aggregation rules requires votes to be normalized.
    pub fn normalized(&self) -> Votes {
        self.votes
            .iter()
            .map(|(uid, vote)| {
                let sum = vote.iter().map(|v| v.1).fold(0.0, |a, x| a + x);

                if sum == 0.0 {
                    (uid.to_owned(), vote.to_owned())
                } else {
                    let normalized = vote
                        .iter()
                        .map(|(to, v)| (to.to_owned(), v / sum))
                        .collect();
                    (uid.to_owned(), normalized)
                }
            })
            .collect()
    }

    /// Some aggregation rules only needs votes that are casted to policies
    pub fn only_policy_voting(&self) -> Votes {
        self.votes
            .iter()
            .map(|(uid, vote)| {
                (
                    *uid,
                    vote.iter()
                        .filter(|(to, _)| !self.delegates.iter().any(|id| &id == to))
                        // TODO: why do I have to do this?
                        .map(|(uuid, value)| (uuid.to_owned(), value.to_owned()))
                        .collect(),
                )
            })
            .collect()
    }
}

/// `Topic` should be convertable to `VoteInfo`s. This will be the information sent to the
/// calculation modules.
impl From<Topic> for VoteInfo {
    fn from(topic: Topic) -> Self {
        let delegates = topic
            .delegates
            .iter()
            .map(|(uuid, _)| uuid.to_owned())
            .collect();

        let policies = topic
            .policies
            .iter()
            .map(|(uuid, _)| uuid.to_owned())
            .collect();

        Self {
            delegates,
            policies,
            votes: topic.votes.to_owned(),
        }
    }
}

impl Topic {
    pub fn votes(&self) -> Votes {
        self.votes.to_owned()
    }

    /// useful for mocking up votes
    pub fn dummy() -> Self {
        let mut topic = Topic::new("dummy", "which fruit");

        let alice = topic.add_new_delegate("alice").unwrap();
        let bob = topic.add_new_delegate("bob").unwrap();
        let charlie = topic.add_new_delegate("charlie").unwrap();

        let apples = topic.add_new_policy("apples").unwrap();
        let bananas = topic.add_new_policy("bananas").unwrap();
        let _oranges = topic.add_new_policy("oranges").unwrap();

        topic.cast_vote_to(&alice, &apples, 1f64);
        topic.cast_vote_to(&bob, &bananas, 1f64);
        topic.cast_vote_to(&charlie, &bananas, 1f64);

        topic
    }

    pub fn get_id_by_name(&self, name: &str) -> Option<Uuid> {
        self.delegates
            .iter()
            .find(|(_, voter_name)| name == *voter_name)
            .and_then(|(id, _name)| Some(id.to_owned()))
    }

    pub fn get_id_by_title(&self, title: &str) -> Option<Uuid> {
        self.delegates
            .iter()
            .find(|(_, pt)| title == *pt)
            .and_then(|(id, _)| Some(id.to_owned()))
    }
}

#[cfg(test)]
mod topic_info_test {

    use super::*;

    #[test]
    fn normalize_votes_no_change_empty() {
        let mut topic = Topic::new("empty", "");
        let initial_votes = topic.votes.to_owned();

        topic.add_new_delegate("alice");
        topic.add_new_policy("apples");
        topic.add_new_policy("bananas");

        let info: VoteInfo = topic.into();

        let normalized = info.normalized();

        assert_eq!(initial_votes, normalized);
    }

    #[test]
    fn normalize_votes() {
        let mut topic = Topic::dummy();

        let alice = topic.get_id_by_name("alice").unwrap();
        let apples = topic.get_id_by_title("apples").unwrap();
        let bananas = topic.get_id_by_title("bananas").unwrap();
        topic.cast_vote_to(&alice, &bananas, 1f64);

        let info: VoteInfo = topic.into();
        let votes = info.normalized();

        let alice_votes = votes.get(&alice).unwrap();
        assert_eq!(alice_votes.get(&apples), alice_votes.get(&bananas));
    }

    #[test]
    fn only_policy() {
        let mut topic = Topic::dummy();

        let alice = topic.get_id_by_name("alice").unwrap();
        let bob = topic.get_id_by_name("bob").unwrap();

        let alice_vote_len = topic
            .votes
            .get(&alice)
            .unwrap()
            .keys()
            .collect::<Vec<&Uuid>>()
            .len();

        topic.cast_vote_to(&alice, &bob, 1f64);

        let info: VoteInfo = topic.into();

        let stripped = info
            .only_policy_voting()
            .get(&alice)
            .unwrap()
            .keys()
            .collect::<Vec<&Uuid>>()
            .len();

        assert_eq!(stripped, alice_vote_len);
    }
}
