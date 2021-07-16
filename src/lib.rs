pub mod rpc;

use async_trait::async_trait;
use bs58::encode;
use futures::executor::block_on;
use futures::future::join3;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// Votes shows how the users votes are handled.
/// The first Uuid must be one assigned to the `delegates` field,
/// yet the second one does not restrict it to `polices`
pub type Votes = BTreeMap<Uuid, BTreeMap<Uuid, f64>>;

/// TopicData is a high level struct that has comprehensive info about the `topic`.
/// It has two purposes:
///   1. a data format that is ready to be used in the front end,
///   2. a format that is saved in a database
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopicData {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    delegates: BTreeMap<Uuid, String>,
    policies: BTreeMap<Uuid, String>,
    votes: Votes,
}

impl TopicData {
    pub fn new(title: &str, description: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: description.to_string(),
            delegates: BTreeMap::new(),
            policies: BTreeMap::new(),
            votes: BTreeMap::new(),
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

    pub fn add_delegate(&mut self, id: &Uuid, nickname: &str) -> bool {
        if self.delegates.iter().any(|(uid, _name)| uid == id) {
            false
        } else {
            self.delegates.insert(id.to_owned(), nickname.to_string());
            true
        }
    }

    pub fn force_add_delegate(&mut self, id: &Uuid, nickname: &str) {
        self.delegates.insert(id.to_owned(), nickname.to_string());
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

    pub fn overwrite_vote_for(&mut self, src: Uuid, vote: BTreeMap<Uuid, f64>) {
        self.votes.insert(src, vote);
    }

    pub fn policies_values(&self) -> BTreeSet<String> {
        self.policies.iter().map(|(_id, v)| v.to_owned()).collect()
    }

    pub fn delegates_values(&self) -> BTreeSet<String> {
        self.delegates.iter().map(|(_id, v)| v.to_owned()).collect()
    }

    pub fn cast_vote_to(&mut self, src: &Uuid, target: &Uuid, value: f64) {
        //TODO : check for illegal votes
        self.votes
            .entry(src.to_owned())
            .or_insert(BTreeMap::new())
            .insert(target.to_owned(), value);
    }
}

/// VoteData is a redacted version of `TopicData`.
/// The calculation modules will not need to know the full
/// information about the topic.
/// This struct strictly defines only the necessary information
/// for aggregating votes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoteData {
    /// notice the data format is different from the one in `TopicData`
    pub delegates: BTreeSet<Uuid>,
    pub policies: BTreeSet<Uuid>,
    pub votes: Votes,
}

impl VoteData {
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

    /// Some aggregation rules only needs votes that are casted to policies
    pub fn only_delegate_voting(&self) -> Votes {
        self.votes
            .iter()
            .map(|(uid, vote)| {
                (
                    *uid,
                    vote.iter()
                        .filter(|(to, _)| !self.policies.iter().any(|id| &id == to))
                        // TODO: why do I have to do this?
                        .map(|(uuid, value)| (uuid.to_owned(), value.to_owned()))
                        .collect(),
                )
            })
            .collect()
    }

    pub fn hash_sync(&self) -> Vec<u8> {
        block_on(self.hash())
    }

    /// used to calculate the uniqueness of votes
    pub async fn hash(&self) -> Vec<u8> {
        let d_hash = async move {
            let mut hasher = Sha256::new();
            for d in self.delegates.to_owned() {
                hasher.update(&d.as_bytes());
            }
            hasher.finalize().as_slice().to_owned()
        };

        let p_hash = async move {
            let mut hasher = Sha256::new();
            for p in self.delegates.to_owned() {
                hasher.update(&p.as_bytes());
            }
            hasher.finalize().as_slice().to_owned()
        };

        let v_hash = async move {
            let mut hasher = Sha256::new();
            for (voter, vote) in self.votes.to_owned() {
                hasher.update(voter.as_bytes());
                for (to, value) in vote {
                    hasher.update(to.as_bytes());
                    hasher.update(value.to_be_bytes());
                }
            }
            hasher.finalize().as_slice().to_owned()
        };

        let (delegates, policies, vote) = join3(d_hash, p_hash, v_hash).await;

        let mut hasher = Sha256::new();
        hasher.update(&delegates.as_slice());
        hasher.update(&policies.as_slice());
        hasher.update(&vote.as_slice());

        hasher.finalize().as_slice().to_owned()
    }
}

/// `TopicData` should be convertable to `VoteData`s. This will be the information sent to the
/// calculation modules.
impl From<TopicData> for VoteData {
    fn from(topic: TopicData) -> Self {
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

impl TopicData {
    pub fn votes(&self) -> Votes {
        self.votes.to_owned()
    }

    /// useful for mocking up votes
    pub fn dummy() -> Self {
        let mut topic = TopicData::new("dummy", "which fruit");

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
        self.policies
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
        let mut topic = TopicData::new("empty", "");
        let initial_votes = topic.votes.to_owned();

        topic.add_new_delegate("alice");
        topic.add_new_policy("apples");
        topic.add_new_policy("bananas");

        let info: VoteData = topic.into();

        let normalized = info.normalized();

        assert_eq!(initial_votes, normalized);
    }

    #[test]
    fn normalize_votes() {
        let mut topic = TopicData::dummy();

        let alice = topic.get_id_by_name("alice").unwrap();
        let apples = topic.get_id_by_title("apples").unwrap();
        let bananas = topic.get_id_by_title("bananas").unwrap();
        topic.cast_vote_to(&alice, &bananas, 1f64);

        let info: VoteData = topic.into();
        let votes = info.normalized();

        let alice_votes = votes.get(&alice).unwrap();
        assert_eq!(alice_votes.get(&apples), alice_votes.get(&bananas));
    }

    #[test]
    fn only_policy() {
        let mut topic = TopicData::dummy();

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

        let info: VoteData = topic.into();

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

#[async_trait]
pub trait AggregationRule {
    async fn calculate(vote: VoteData) -> Value;
}
