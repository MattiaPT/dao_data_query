use std::{error::Error as err, fs, path};

use chrono::{TimeZone, Utc};
use itertools::Itertools;
use log::info;

pub mod utils;
use utils::*;

macro_rules! format_num {
    ($x:expr) => {
        if $x != 0 {
            format!("{}", $x)
        } else {
            String::from("-")
        }
    };
}

fn main() -> Result<(), Box<dyn err>> {
    env_logger::init();

    let mut data_container: Vec<(String, ProposalResponse)> = Vec::new();

    for resp_file in fs::read_dir(RESP_DIR).unwrap() {
        let dir_entry = resp_file?;
        if dir_entry.path() == path::Path::new(RAW_PROTOCOLS) {
            info!("Skipping {}", dir_entry.path().display());
            continue;
        }
        info!("Deserializing {}", dir_entry.path().display());
        let content = fs::read_to_string(dir_entry.path())?;

        let response: ProposalResponse = serde_json::from_str(&content)?;
        data_container.push((dir_entry.file_name().into_string().unwrap(), response));
    }

    print_proposal_tables_1(&data_container)?;
    // print_proposal_table_2(&data_container)?;

    Ok(())
}

fn print_proposal_tables_1(
    data_container: &Vec<(String, ProposalResponse)>,
) -> Result<(), Box<dyn err>> {
    let data_filtered: &Vec<&(String, ProposalResponse)> = &data_container
        .iter()
        .filter(|(_, proposal)| {
            proposal.data.is_some() && proposal.data.clone().unwrap().len() >= 5
        })
        .collect();

    for chunk in &data_filtered.into_iter().chunks(70) {
        println!("");
        println!(
            "{:<30} | {:<10} | {:<20} | {:<20} | {:<15} | {:<9} | {:<15} | {:<12} | {:<12}",
            "DAO",
            "#proposals",
            "(P/A/Q/E/X/C)",
            "(O/U/S/T/O/A)",
            "avg. voters",
            "#snapshot",
            "avg. voters",
            "oldest prop.",
            "newest prop."
        );

        for (dao_name, boardroom) in chunk {
            let data = boardroom.data.clone().unwrap();

            let (
                mut num_pending,
                mut num_active,
                mut num_queued,
                mut num_executed,
                mut num_canceled,
                mut num_closed,
                mut num_onchain,
                mut num_o_upgrade,
                mut num_o_secondary,
                mut num_o_tertiary,
                mut num_o_optimism,
                mut num_o_arbitrum,
                mut avg_onchain_voters,
                mut num_snapshot,
                mut avg_snapshot_voters,
            ) = (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0.0, 0, 0.0);

            let num_proposals = data.len();

            data.iter().for_each(|proposal| {
                match proposal.adapter {
                    Adapter::Onchain => num_onchain += 1,
                    Adapter::OnchainUpgrade => num_o_upgrade += 1,
                    Adapter::OnchainSecondary => num_o_secondary += 1,
                    Adapter::OnchainTertiary => num_o_tertiary += 1,
                    Adapter::OnchainOptimism => num_o_optimism += 1,
                    Adapter::OnchainArbitrum => num_o_arbitrum += 1,
                    Adapter::Snapshot => {
                        num_snapshot += 1;
                        avg_snapshot_voters += proposal.total_votes as f64;
                    }
                    Adapter::Archive => (),
                };
                match proposal.current_state {
                    ProposalState::PENDING => num_pending += 1,
                    ProposalState::ACTIVE => num_active += 1,
                    ProposalState::QUEUED => num_queued += 1,
                    ProposalState::EXECUTED => num_executed += 1,
                    ProposalState::CANCELED => num_canceled += 1,
                    ProposalState::CLOSED => num_closed += 1,
                };

                match proposal.adapter {
                    Adapter::Snapshot => (),
                    Adapter::Archive => (),
                    _ => {
                        num_onchain += 1;
                        avg_onchain_voters += proposal.total_votes as f64;
                    }
                }
            });

            let oldest_proposal = if data.len() != 0 {
                Utc.timestamp_opt(
                    data.iter()
                        .map(|proposal| proposal.start_timestamp)
                        .min()
                        .unwrap()
                        .into(),
                    0,
                )
                .unwrap()
                .format("%Y-%m-%d")
                .to_string()
            } else {
                "-".to_string()
            };
            let newest_proposal = if data.len() != 0 {
                Utc.timestamp_opt(
                    data.iter()
                        .map(|proposal| proposal.start_timestamp)
                        .max()
                        .unwrap()
                        .into(),
                    0,
                )
                .unwrap()
                .format("%Y-%m-%d")
                .to_string()
            } else {
                "-".to_string()
            };

            let avg_snapshot_voters = if num_snapshot == 0 {
                "-".to_string()
            } else {
                format!("{:.2}", avg_snapshot_voters / (num_snapshot as f64))
            };

            let avg_onchain_voters = if num_onchain == 0 {
                "-".to_string()
            } else {
                format!("{:.2}", avg_onchain_voters / (num_onchain as f64))
            };

            println!(
                "{:<30} | {:<10} | {:<20} | {:<20} | {:<15} | {:<9} | {:<15} | {:<12} | {:<12}",
                dao_name,
                num_proposals,
                format!(
                    "({}/{}/{}/{}/{}/{})",
                    format_num!(num_pending),
                    format_num!(num_active),
                    format_num!(num_queued),
                    format_num!(num_executed),
                    format_num!(num_canceled),
                    format_num!(num_closed)
                ),
                format!(
                    "({}/{}/{}/{}/{}/{})",
                    format_num!(num_onchain),
                    format_num!(num_o_upgrade),
                    format_num!(num_o_secondary),
                    format_num!(num_o_tertiary),
                    format_num!(num_o_optimism),
                    format_num!(num_o_arbitrum)
                ),
                avg_onchain_voters,
                format_num!(num_snapshot),
                avg_snapshot_voters,
                oldest_proposal,
                newest_proposal,
            );
        }
    }

    Ok(())
}

fn print_proposal_table_2(
    data_container: &Vec<(String, ProposalResponse)>,
) -> Result<(), Box<dyn err>> {
    println!("{:<20} | {:<71} | {:<60}", "DAO", "snapshot", "onchain",);
    println!(
        "{:<20} | {:<20} | {:<20} | {:<25} | {:<20} | {:<20} | {:<20}",
        "",
        "avg. quorum",
        "(s) avg. distinct voters",
        "avg. w votes",
        "avg. quorum",
        "avg. distinct voters",
        "avg. w votes"
    );

    for (dao_name, boardroom) in data_container.iter() {
        if boardroom.data.is_none() {
            continue;
        }

        let data = boardroom.data.clone().unwrap();

        let (
            mut avg_snapshot_quorum,
            mut avg_snapshot_voters,
            mut avg_snapshot_winning_votes,
            mut avg_onchain_quorum,
            mut avg_onchain_voters,
            mut avg_onchain_winning_votes,
        ) = (0, 0, 0.0, 0, 0, 0.0);
        let (mut num_snapshot, mut num_onchain) = (0, 0);

        data.iter().for_each(|proposal| match proposal.adapter {
            Adapter::Snapshot => {
                num_snapshot += 1;
                avg_snapshot_quorum += proposal.quorum;
                avg_snapshot_voters += proposal.total_votes;

                avg_snapshot_winning_votes += proposal
                    .results
                    .iter()
                    .map(|res| res.total)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or_default()
            }
            _ => {
                num_onchain += 1;
                avg_onchain_quorum += proposal.quorum;
                avg_onchain_voters += proposal.total_votes;

                avg_onchain_winning_votes += proposal
                    .results
                    .iter()
                    .map(|res| res.total)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or_default()
            }
        });

        (
            avg_snapshot_quorum,
            avg_snapshot_voters,
            avg_snapshot_winning_votes,
        ) = if num_snapshot == 0 {
            (0, 0, 0.0)
        } else {
            (
                avg_snapshot_quorum / num_snapshot,
                avg_snapshot_voters / num_snapshot,
                avg_snapshot_winning_votes / (num_snapshot as f64),
            )
        };

        (
            avg_onchain_quorum,
            avg_onchain_voters,
            avg_onchain_winning_votes,
        ) = if num_onchain == 0 {
            (0, 0, 0.0)
        } else {
            (
                avg_onchain_quorum / num_onchain,
                avg_onchain_voters / num_onchain,
                avg_onchain_winning_votes / (num_onchain as f64),
            )
        };

        if data.len() < 5 {
            continue;
        }

        println!(
            "{:<20} | {:<20} | {:<20} | {:<25} | {:<20} | {:<20} | {:<20}",
            dao_name,
            avg_snapshot_quorum,
            avg_snapshot_voters,
            avg_snapshot_winning_votes,
            avg_onchain_quorum,
            avg_onchain_voters,
            avg_onchain_winning_votes
        );
    }

    Ok(())
}
