use serde::{de, Deserialize, Deserializer};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

/* ------- SETTINGS ------- */
pub const QUERY_DIR: &str = "queries";
pub const RESP_DIR: &str = "data/responses";

pub const SUBGRAPH_IDS: &str = "data/subgraph_ids.csv";
/* ------------------------ */

/* ------- STRUCTS -------- */

#[derive(Deserialize, Serialize, Debug)]
pub struct SubgraphCSVRow {
    pub name: String,
    pub subgraph_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum ProposalState {
    PENDING,
    ACTIVE,
    QUEUED,
    EXECUTED,
    CANCELED,
}

/* Proposal Query Response */
#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalData {
    pub id: String,
    #[serde(deserialize_with = "deserialize_state")]
    pub state: ProposalState,
    #[serde(alias = "creationTime", deserialize_with = "deserialize_u32")]
    pub creation_time: u32,
    #[serde(alias = "abstainDelegateVotes", deserialize_with = "deserialize_u128")]
    pub abstain_delegate_votes: u128,
    #[serde(alias = "abstainWeightedVotes", deserialize_with = "deserialize_u128")]
    pub abstain_weighted_votes: u128,
    #[serde(alias = "againstDelegateVotes", deserialize_with = "deserialize_u128")]
    pub against_delegate_votes: u128,
    #[serde(alias = "againstWeightedVotes", deserialize_with = "deserialize_u128")]
    pub against_weighted_votes: u128,
    #[serde(alias = "forDelegateVotes", deserialize_with = "deserialize_u128")]
    pub for_delegate_votes: u128,
    #[serde(alias = "forWeightedVotes", deserialize_with = "deserialize_u128")]
    pub for_weighted_votes: u128,
    #[serde(alias = "totalDelegateVotes", deserialize_with = "deserialize_u128")]
    pub total_delegate_votes: u128,
    #[serde(alias = "totalWeightedVotes", deserialize_with = "deserialize_u128")]
    pub total_weighted_votes: u128,
    #[serde(alias = "quorumVotes", deserialize_with = "deserialize_u128")]
    pub quorum_votes: u128,
    #[serde(alias = "delegatesAtStart", deserialize_with = "deserialize_u128")]
    pub delegates_at_start: u128,
    #[serde(alias = "tokenHoldersAtStart", deserialize_with = "deserialize_u128")]
    pub token_holders_at_start: u128,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GovernanceData {
    #[serde(deserialize_with = "deserialize_u32")]
    pub proposals: u32,
    #[serde(alias = "totalTokenSupply", deserialize_with = "deserialize_u128")]
    pub total_token_supply: u128,
    #[serde(alias = "delegatedVotesRaw", deserialize_with = "deserialize_u128")]
    pub delegated_votes_raw: u128,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalQueryResponse {
    pub governances: Vec<GovernanceData>,
    pub proposals: Vec<ProposalData>,
}

/* ----------------------- */

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum Data {
    Query(ProposalQueryResponse),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GraphResponse {
    pub data: Data,
}

/* ------------------------ */

/* ------- FUNCTIONS -------- */
fn deserialize_u32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse::<u32>().map_err(de::Error::custom)?,
        _ => return Err(de::Error::custom("wrong type for deserializer")),
    })
}

fn deserialize_u128<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u128, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse::<u128>().map_err(de::Error::custom)?,
        _ => return Err(de::Error::custom("wrong type for deserializer")),
    })
}

fn deserialize_state<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<ProposalState, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => match s.as_str() {
            "PENDING" => ProposalState::PENDING,
            "ACTIVE" => ProposalState::ACTIVE,
            "QUEUED" => ProposalState::QUEUED,
            "EXECUTED" => ProposalState::EXECUTED,
            "CANCELED" => ProposalState::CANCELED,
            _ => return Err(de::Error::custom("Unknown proposal state")),
        },
        _ => return Err(de::Error::custom("Wrong type for deserializer")),
    })
}

/* -------------------------- */
