// Note: The current setup avoids the subgraphs of the Rarible, Ooki and OUSD DAOs, because
// the data does not fit in the rest

use std::{env, error::Error as err, fs, io::{prelude::*, Read}};

use chrono::{TimeZone, Utc};
use log::info;
use reqwest::{Client, Error, Response};

pub mod utils;
use utils::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn err>> {
    env_logger::init();

    info!("Fetching subgraph ");

    let subgraphs: Vec<SubgraphCSVRow> = csv::Reader::from_reader(fs::File::open(SUBGRAPH_IDS)?)
        .records()
        .map(|result| result.unwrap().deserialize::<SubgraphCSVRow>(None).unwrap())
        .collect();

    let client = Client::new();
    let mut open_queries: Vec<(String, &str, Response)> = Vec::new();

    for (name, mut file) in get_files(QUERY_DIR).await {
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let content_parts: Vec<&str> = content.split("\n\n").collect();
        let (query_url, query): (&str, &str) = (content_parts[0], content_parts[1]);
        for subgraph in subgraphs.iter() {
            let open_query = initiate_query(
                &client,
                query_url.replace("{subgraph-id}", &subgraph.subgraph_id),
                query.replace("\n", ""),
            )
            .await?;
            open_queries.push((name.clone(), &subgraph.name, open_query));
            info!("Successfully sent query: {} for {}", &name, &subgraph.name);
        }
    }

    let mut data_container: Vec<(&str, GraphResponse)> = Vec::new();

    for (filename, dao_name, response) in open_queries {
        let resp_body = response.text().await?;
        let path = RESP_DIR.to_owned() + "/" + dao_name + "_" + &filename;
        let mut file = fs::File::create(&path)?;
        let deserialized_response: GraphResponse = serde_json::from_str(&resp_body)?;
        file.write_all(format!("{:#?}", deserialized_response).as_bytes())?;
        data_container.push((dao_name, deserialized_response));
        info!("Wrote response to {}", &path);
    }

    create_table_a(&data_container).await;
    create_table_b(&data_container).await;

    Ok(())
}

// INITIATE QUERY
// send a post request to the given url using the passed client
// and attach the passed query
async fn initiate_query(client: &Client, url: String, query: String) -> Result<Response, Error> {
    client
        .post(url.replace("{api-key}", &env::var("GRAPH_API_KEY").unwrap()))
        .body(format!(
            r#"{{"query": "{}", "operationName": "Subgraphs", "variables": {{}}}}"#,
            query
        ))
        .send()
        .await
}

// GET FILES
// returns a tuple vector with (filename, file handle) given a path
async fn get_files(rel_dir_path: &str) -> Vec<(String, fs::File)> {
    fs::read_dir(rel_dir_path)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|dir_entry| {
            (
                dir_entry.file_name().into_string().unwrap(),
                fs::File::open(dir_entry.path()),
            )
        })
        .filter(|(_, file)| file.is_ok())
        .map(|(name, file)| (name, file.unwrap()))
        .collect()
}

// GENERATE TABLES
// returns a table of the following structure:
// dao_name | #props. | #pending props. | #executed props. | #queued props. | #active props. | #canceled props. | oldes prop. | newest prop. | avg. voting participation (num del) | avg_voting_participation_of_delegates
async fn create_table_a(data_container: &Vec<(&str, GraphResponse)>) {
    println!(
        "{:<15} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10}",
        "DAO", "#proposals", "#pending", "#active", "#queued", "#executed", "#canceled", "oldest", "newest", "avg. del");
    for (dao_name, response) in data_container.iter() {
        match &response.data {
            Data::Query(response_data) => {
                let num_proposals = response_data.governances[0].proposals as u128;
                let (
                    mut num_pending,
                    mut num_executed,
                    mut num_queued,
                    mut num_active,
                    mut num_canceled,
                ) = (0, 0, 0, 0, 0);
                response_data
                    .proposals
                    .iter()
                    .for_each(|proposal| match proposal.state {
                        ProposalState::PENDING => num_pending += 1,
                        ProposalState::EXECUTED => num_executed += 1,
                        ProposalState::QUEUED => num_queued += 1,
                        ProposalState::ACTIVE => num_active += 1,
                        ProposalState::CANCELED => num_canceled += 1,
                    });
                let oldest_recent_proposal = Utc
                    .timestamp_opt(
                        response_data
                            .proposals
                            .iter()
                            .map(|proposal| proposal.creation_time)
                            .min()
                            .unwrap() as i64,
                        0,
                    )
                    .unwrap();
                let most_recent_proposal = Utc
                    .timestamp_opt(
                        response_data
                            .proposals
                            .iter()
                            .map(|proposal| proposal.creation_time)
                            .max()
                            .unwrap() as i64,
                        0,
                    )
                    .unwrap();
                // let total_token_supply = response_data.governances[0].total_token_supply;
                let avg_delegates: u128 = response_data.proposals.iter().map(|proposal| proposal.delegates_at_start / num_proposals).sum();

                println!(
                    "{:<15} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} ",
                    dao_name,
                    num_proposals,
                    num_pending,
                    num_active,
                    num_queued,
                    num_executed,
                    num_canceled,
                    oldest_recent_proposal.format("%Y-%m-%d").to_string(),
                    most_recent_proposal.format("%Y-%m-%d").to_string(),
                    avg_delegates,
                )
            }
        }
    }
}

// returns a table of the following structure:
// dao_name | avg. winning votes |
async fn create_table_b(data_container: &Vec<(&str, GraphResponse)>) {
    println!(
        "{:<15} | {:<40} | {:<40} | {:<30} | {:<30} | {:<30}",
        "DAO", "avg. participation (by voting power)", "avg. participation (by distinct del)", "avg quorum", "avg. winning votes", "avg. winning voters"
    );
    for (dao_name, response) in data_container.iter() {
        match &response.data {
            Data::Query(response_data) => {
                let mut num_finished_votes = 0;
                let mut winning_vp: u128 = 0;
                let mut distinct_winning_voters = 0;
                let num_proposals = response_data.governances[0].proposals;

                response_data.proposals.iter().for_each(|proposal| match proposal.state {
                    ProposalState::PENDING => (),
                    ProposalState::ACTIVE => (),
                    ProposalState::QUEUED => {
                        num_finished_votes += 1;
                        winning_vp += proposal.for_weighted_votes;
                        distinct_winning_voters += proposal.for_delegate_votes;
                    },
                    ProposalState::EXECUTED => {
                        num_finished_votes += 1;
                        winning_vp += proposal.for_weighted_votes;
                        distinct_winning_voters += proposal.for_delegate_votes;
                    },
                    ProposalState::CANCELED => {
                        num_finished_votes += 1;
                        winning_vp += proposal.against_weighted_votes;
                        distinct_winning_voters += proposal.against_delegate_votes;
                    },
                });

                let avg_voting_participation: f64 = response_data
                    .proposals
                    .iter()
                    .map(|proposal| {
                        (proposal.total_weighted_votes as f64)
                            / (response_data.governances[0].delegated_votes_raw as f64)
                            / (num_proposals as f64)
                    })
                    .sum();
                let delegates_voting_participation: f64 = response_data
                    .proposals
                    .iter()
                    .map(|proposal| {
                        if proposal.delegates_at_start == 0 {
                            0.0
                        } else {
                            (proposal.total_delegate_votes as f64)
                                / (proposal.delegates_at_start as f64)
                                / (num_proposals as f64)
                        }
                    })
                    .sum();
                let avg_quorum: f64 = response_data
                    .proposals
                    .iter()
                    .map(|proposal| {
                        (proposal.quorum_votes as f64)
                            / (num_proposals as f64)
                            / (u128::pow(10, 18) as f64)
                    })
                    .sum();
                println!(
                    "{:<15} | {:<40} | {:<40} | {:<30} | {:<30} | {:<30}",
                    dao_name,
                    format_percentage(avg_voting_participation, 2),
                    format_percentage(delegates_voting_participation, 2),
                    avg_quorum as u32,
                    winning_vp / num_finished_votes / u128::pow(10, 18),
                    distinct_winning_voters / num_finished_votes
                );
            }
        }
    }
}

fn format_percentage(n: f64, dec: u32) -> String {
    let perc = n * 100.0;
    let whole = perc as i32;
    let frac = ((perc % 1.0) * (i32::pow(10, dec) as f64)) as i32;
    format!("{:>2}.{:0<2}%", whole, frac)
}
