use serde::{de, Deserialize, Deserializer, Serializer};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

/* ------- SETTINGS ------- */
pub const CNAMES: &str = "data/cnames.csv";

pub const CNAMES_URL: &str = "https://api.boardroom.info/v1/protocols?limit=10000&key={api-key}";
pub const PROPOSAL_URL: &str =
    "https://api.boardroom.info/v1/protocols/{cname}/proposals?limit=10000&key={api-key}";

pub const RESP_DIR: &str = "data/responses";
pub const RAW_PROTOCOLS: &str = "data/responses/protocols";
/* ------------------------ */

/* ------- STRUCTS -------- */

#[derive(Deserialize, Serialize, Debug)]
pub struct CNAMECSVRow {
    pub dao: String,
    pub cname: String,
}

/* CNAMES Query Response */

#[derive(Deserialize, Serialize, Debug)]
pub struct Icon {
    pub adapter: String,
    pub size: String,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MarketPrice {
    pub currency: String,
    pub price: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Token {
    pub adapter: String,
    pub symbol: String,
    pub network: String,
    #[serde(rename = "contractAddress")]
    pub contract_address: String,
    #[serde(rename = "totalSupply")]
    pub total_supply: Option<u128>,
    #[serde(rename = "circulatingSupply")]
    pub circulating_supply: Option<u128>,
    #[serde(rename = "maxSupply")]
    pub max_supply: Option<u128>,
    #[serde(rename = "marketPrices")]
    pub market_prices: Option<Vec<MarketPrice>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProtocolObject {
    pub cname: String,
    pub name: String,
    pub categories: Vec<String>,
    #[serde(rename = "isEnabled")]
    pub is_enabled: bool,
    #[serde(rename = "activeOnWebsite")]
    pub active_on_website: bool,
    #[serde(rename = "totalProposals")]
    pub total_proposals: u128,
    #[serde(rename = "totalVotes")]
    pub total_votes: u128,
    #[serde(rename = "uniqueVoters")]
    pub unique_voters: u128,
    pub icons: Option<Vec<Icon>>,
    pub tokens: Option<Vec<Token>>,
    #[serde(rename = "type")]
    pub protocol_type: String,
    #[serde(skip_deserializing, rename = "associatedProtocols")]
    pub associated_protocols: String,
    #[serde(skip_deserializing, rename = "associatedAddresses")]
    pub associated_addresses: String,
    #[serde(rename = "delegationSupport")]
    pub delegation_support: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProtocolResponse {
    pub data: Vec<ProtocolObject>,
}

/* Proposal Query Response */

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Adapter {
    Onchain,
    OnchainUpgrade,
    OnchainSecondary,
    OnchainTertiary,
    OnchainOptimism,
    OnchainArbitrum,
    Snapshot,
    Archive,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ProposalState {
    PENDING,
    QUEUED,
    ACTIVE,
    EXECUTED,
    CANCELED,
    CLOSED,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ProposalType {
    Basic,
    SingleChoice,
    RankedChoice,
    Approval,
    Weighted,
    Quadratic,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TimeObject {
    pub timestamp: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChoiceResult {
    #[serde(deserialize_with = "deserialize_f64")]
    pub total: f64,
    #[serde(deserialize_with = "deserialize_i32")]
    pub choice: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockObject {
    #[serde(rename = "blockNumber")]
    pub block_number: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Event {
    pub time: BlockObject,
    #[serde(
        deserialize_with = "deserialize_state",
        serialize_with = "serialize_state"
    )]
    pub event: ProposalState,
    pub timestamp: u32,
    #[serde(rename = "txHash")]
    pub tx_hash: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct IndexedResult {
    #[serde(deserialize_with = "deserialize_f64")]
    pub total: f64,
    #[serde(deserialize_with = "deserialize_u32")]
    pub choice: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProposalObject {
    #[serde(rename = "refId")]
    pub ref_id: String,
    pub id: String,
    pub title: String,
    pub content: String,
    pub protocol: String,
    #[serde(
        deserialize_with = "deserialize_adapter",
        serialize_with = "serialize_adapter"
    )]
    pub adapter: Adapter,
    pub proposer: String,
    #[serde(rename = "totalVotes")]
    pub total_votes: u128,
    #[serde(rename = "blockNumber")]
    pub block_number: u128,
    #[serde(rename = "externalUrl")]
    pub external_url: Option<String>,
    #[serde(rename = "startTime")]
    pub start_time: TimeObject,
    #[serde(rename = "endTime")]
    pub end_time: TimeObject,
    #[serde(rename = "startTimestamp", deserialize_with = "deserialize_u32")]
    pub start_timestamp: u32,
    #[serde(rename = "endTimestamp", deserialize_with = "deserialize_u32")]
    pub end_timestamp: u32,
    #[serde(
        rename = "currentState",
        deserialize_with = "deserialize_state",
        serialize_with = "serialize_state"
    )]
    pub current_state: ProposalState,
    pub choices: Vec<String>,
    pub results: Vec<ChoiceResult>,
    pub events: Vec<Event>,
    #[serde(
        rename = "type",
        deserialize_with = "deserialize_type",
        serialize_with = "serialize_type"
    )]
    pub proposal_type: Option<ProposalType>,
    #[serde(rename = "indexedResult")]
    pub indexed_result: Option<Vec<Option<IndexedResult>>>, // not deserializing data, as already contained in results field
    pub summary: Option<String>,
    pub privacy: Option<String>,
    #[serde(rename = "indexedAt")]
    pub indexed_at: u128,
    #[serde(rename = "txHash")]
    pub tx_hash: String,
    pub quorum: u128,
    pub flagged: Option<String>,
    #[serde(skip_deserializing, rename = "executionArgs")] // empty struct can be ignored
    pub execution_args: String,
    #[serde(skip_deserializing)] // empty struct can be ignored
    pub executables: String,
    #[serde(rename = "chainId")]
    pub chain_id: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalResponse {
    pub data: Option<Vec<ProposalObject>>,
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

/* ----------------------- */

fn deserialize_adapter<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Adapter, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => match s.as_str() {
            "onchain" => Adapter::Onchain,
            "onchain-upgrade" => Adapter::OnchainUpgrade,
            "onchain-secondary" => Adapter::OnchainSecondary,
            "onchain-tertiary" => Adapter::OnchainTertiary,
            "onchain-optimism" => Adapter::OnchainOptimism,
            "onchain-arbitrum" => Adapter::OnchainArbitrum,
            "snapshot" => Adapter::Snapshot,
            "archive" => Adapter::Archive,
            "archiveAlpha" => Adapter::Archive,
            e => {
                println!("Other adapter: {}", e);
                return Err(de::Error::custom("Unknown adapter state"));
            }
        },
        _ => return Err(de::Error::custom("Wrong type for deserializer")),
    })
}

fn serialize_adapter<S: Serializer>(val: &Adapter, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(match val {
        Adapter::Onchain => "onchain",
        Adapter::OnchainUpgrade => "onchain-upgrade",
        Adapter::OnchainSecondary => "onchain-secondary",
        Adapter::OnchainTertiary => "onchain-tertiary",
        Adapter::OnchainOptimism => "onchain-optimism",
        Adapter::OnchainArbitrum => "onchain-arbitrum",
        Adapter::Snapshot => "snapshot",
        Adapter::Archive => "archive",
    })
}

fn deserialize_state<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<ProposalState, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => match s.as_str() {
            "pending" => ProposalState::PENDING,
            "queued" => ProposalState::QUEUED,
            "active" => ProposalState::ACTIVE,
            "executed" => ProposalState::EXECUTED,
            "canceled" => ProposalState::CANCELED,
            "closed" => ProposalState::CLOSED,
            e => {
                println!("Other proposal state: {}", e);
                return Err(de::Error::custom("Unknown proposal state"));
            }
        },
        _ => return Err(de::Error::custom("Wrong type for deserializer")),
    })
}

fn serialize_state<S: Serializer>(val: &ProposalState, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(match val {
        ProposalState::PENDING => "pending",
        ProposalState::QUEUED => "queued",
        ProposalState::ACTIVE => "active",
        ProposalState::EXECUTED => "executed",
        ProposalState::CANCELED => "canceled",
        ProposalState::CLOSED => "closed",
    })
}

fn deserialize_type<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ProposalType>, D::Error> {
    Ok(
        match Option::<String>::deserialize(deserializer)?.as_deref() {
            Some("basic") => Some(ProposalType::Basic),
            Some("single-choice") => Some(ProposalType::SingleChoice),
            Some("ranked-choice") => Some(ProposalType::RankedChoice),
            Some("approvalVoting") => Some(ProposalType::Approval),
            Some("approval") => Some(ProposalType::Approval),
            Some("optimisticApproval") => Some(ProposalType::Approval),
            Some("weighted") => Some(ProposalType::Weighted),
            Some("quadratic") => Some(ProposalType::Quadratic),
            Some(e) => {
                println!("Other proposal type: {}", e);
                return Err(de::Error::custom("Unknown proposal type"));
            }
            None => None,
        },
    )
}

fn serialize_type<S: Serializer>(
    val: &Option<ProposalType>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match val {
        Some(val) => serializer.serialize_str(match val {
            ProposalType::Basic => "basic",
            ProposalType::SingleChoice => "single-choice",
            ProposalType::RankedChoice => "ranked-choice",
            ProposalType::Approval => "approval",
            ProposalType::Weighted => "weighted",
            ProposalType::Quadratic => "quadratic",
        }),
        None => serializer.serialize_none(),
    }
}

fn deserialize_u32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse::<u32>().map_err(de::Error::custom)?,
        Value::Number(i) => i.as_u64().unwrap() as u32,
        _ => return Err(de::Error::custom("wrong type for deserializer")),
    })
}

fn deserialize_i32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i32, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse::<i32>().map_err(de::Error::custom)?,
        Value::Number(i) => i.as_i64().unwrap() as i32,
        _ => return Err(de::Error::custom("wrong type for deserializer")),
    })
}

fn deserialize_f64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse::<f64>().map_err(de::Error::custom)?,
        Value::Number(f) => f.as_f64().unwrap(),
        _ => return Err(de::Error::custom("wrong type for deserializer")),
    })
}
