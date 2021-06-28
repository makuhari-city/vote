use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

pub type Delegate = (Uuid, String);
pub type Policy = (Uuid, String);
pub type Votes = HashMap<Uuid, Vec<(Uuid, f64)>>;

// normalizes votes among users votes
pub fn normalize(votes: &Votes) -> Votes {
    votes
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopicInfo {
    pub title: String,
    pub id: Uuid,
    pub votes: Votes,
    pub delegates: Vec<Delegate>,
    pub policies: Vec<Policy>,
}

impl TopicInfo {
    pub fn votes(&self) -> Votes {
        self.votes.to_owned()
    }

    pub fn dummy() -> Self {
        let delegates = vec![
            (Uuid::new_v4(), "alice".to_string()),
            (Uuid::new_v4(), "bob".to_string()),
            (Uuid::new_v4(), "charlie".to_string()),
        ];
        let policies = vec![
            (Uuid::new_v4(), "apples".to_string()),
            (Uuid::new_v4(), "bananas".to_string()),
            (Uuid::new_v4(), "orange".to_string()),
        ];
        let votes: Votes = vec![
            (delegates[0].0, vec![(policies[0].0, 1.0)]),
            (delegates[1].0, vec![(policies[1].0, 1.0)]),
            (delegates[2].0, vec![(policies[1].0, 1.0)]),
        ]
        .into_iter()
        .collect();

        Self {
            title: "topic title".to_string(),
            id: Uuid::new_v4(),
            votes,
            delegates,
            policies,
        }
    }

    pub fn get_id_by_name(&self, name: &str) -> Option<Uuid> {
        self.delegates
            .iter()
            .find(|(_, voter_name)| name == voter_name)
            .and_then(|(id, _name)| Some(id.to_owned()))
    }

    pub fn get_id_by_title(&self, policy_title: &str) -> Option<Uuid> {
        self.delegates
            .iter()
            .find(|(_, title)| title == policy_title)
            .and_then(|(id, _)| Some(id.to_owned()))
    }
}

impl TopicInfo {
    pub fn only_policy_voting(&self) -> Votes {
        self.votes
            .iter()
            .map(|(uid, vote)| {
                (
                    *uid,
                    vote.into_iter()
                        .filter(|(to, _)| !self.delegates.iter().any(|(id, _n)| id == to))
                        .cloned()
                        .collect(),
                )
            })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoteMethodResult {
    method: String,
    result: Value,
}

impl VoteMethodResult {
    pub fn to_tuple(&self) -> (String, Value) {
        (self.method.to_string(), self.result.to_owned())
    }
}

#[cfg(test)]
mod topic_info_test {

    use actix_web::middleware::normalize;

    use super::*;

    #[test]
    fn normalize_votes_no_change_empty() {
        let delegates = vec![(Uuid::new_v4(), "alice".to_string())];

        let policies = vec![
            (Uuid::new_v4(), "apple".to_string()),
            (Uuid::new_v4(), "bananas".to_string()),
        ];

        let votes: Votes = HashMap::new();

        let info = TopicInfo {
            id: Uuid::new_v4(),
            title: "normalized".to_string(),
            votes,
            policies,
            delegates,
        };

        let normalized = normalize(&info.votes);

        assert_eq!(info.votes, normalized);
    }

    #[test]
    fn normalize_votes() {
        let delegates = vec![(Uuid::new_v4(), "alice".to_string())];
        let alice = delegates[0].to_owned();

        let policies = vec![
            (Uuid::new_v4(), "apple".to_string()),
            (Uuid::new_v4(), "bananas".to_string()),
        ];

        let votes: Votes = vec![(
            delegates[0].0,
            vec![(policies[0].0, 1.0), (policies[1].0, 1.0)],
        )]
        .into_iter()
        .collect();

        let info = TopicInfo {
            id: Uuid::new_v4(),
            title: "normalized".to_string(),
            votes,
            policies,
            delegates,
        };

        let normalized = normalize(&info.votes);

        let alices_vote = normalized.get(&alice.0).unwrap();

        assert_eq!(alices_vote[0].1, 0.5);
        assert_eq!(alices_vote[0].1, alices_vote[1].1);
    }

    #[test]
    fn only_policy() {
        let delegates = vec![
            (Uuid::new_v4(), "alice".to_string()),
            (Uuid::new_v4(), "bob".to_string()),
        ];

        let policies = vec![(Uuid::new_v4(), "apple".to_string())];

        let only_policies: Votes = vec![
            (
                delegates[0].0.to_owned(),
                vec![(policies[0].0.to_owned(), 1.0)],
            ),
            (delegates[1].0.to_owned(), vec![]),
        ]
        .into_iter()
        .collect();

        let votes: Votes = vec![
            (
                delegates[0].0,
                vec![(policies[0].0, 1.0), (delegates[1].0, 2.0)],
            ),
            (delegates[1].0, vec![(delegates[0].0, 1.0)]),
        ]
        .into_iter()
        .collect();

        let info = TopicInfo {
            title: "people vote for people".to_string(),
            id: Uuid::new_v4(),
            delegates,
            policies,
            votes,
        };

        let stripped = info.only_policy_voting();

        assert_eq!(stripped, only_policies)
    }
}
