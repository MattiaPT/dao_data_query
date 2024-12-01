use std::{env, error::Error as err, fs, io::prelude::*, path};

use log::info;
use reqwest::{Client, Error, Response};

pub mod utils;
use utils::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn err>> {
    env_logger::init();

    let cname_rows: Vec<CNAMECSVRow> = csv::Reader::from_reader(fs::File::open(CNAMES)?)
        .records()
        .map(|result| result.unwrap().deserialize::<CNAMECSVRow>(None).unwrap())
        .collect();

    let client = Client::new();
    let mut open_queries: Vec<(&CNAMECSVRow, Response)> = Vec::new();

    for row in cname_rows.iter() {
        if path::Path::new(RESP_DIR).join(&row.dao).exists() {
            info!("Skipping {}", &row.dao);
            continue;
        }
        let open_query =
            initiate_query(&client, PROPOSAL_URL.replace("{cname}", &row.cname)).await?;
        open_queries.push((&row, open_query));
        info!("Successfully sent query for {}", &row.dao);
    }

    for (row, response) in open_queries {
        let resp_body = response.text().await?;
        let path = RESP_DIR.to_owned() + "/" + &row.dao;
        let mut file = fs::File::create(&path)?;
        let mut deserialized_response: ProposalResponse = serde_json::from_str(&resp_body)?;

        let mut all_data = deserialized_response.data.take().unwrap_or_default();

        while deserialized_response.next_cursor.is_some() {
            let des_response: ProposalResponse = serde_json::from_str(
                &initiate_query(
                    &client,
                    PROPOSAL_URL.replace("{cname}", &row.cname).to_owned()
                        + "&cursor="
                        + &deserialized_response.next_cursor.clone().unwrap(),
                )
                .await?
                .text()
                .await?,
            )?;

            if let Some(mut data) = des_response.data {
                all_data.append(&mut data);
            }
            deserialized_response.next_cursor = des_response.next_cursor;
        }

        deserialized_response.data = Some(all_data);

        file.write_all(serde_json::to_string_pretty(&deserialized_response)?.as_bytes())?;
        info!("Wrote response to {}", &path);
    }

    Ok(())
}

// INITIATE QUERY
// send a post request to the given url using the passed client
// and attach the passed query
async fn initiate_query(client: &Client, url: String) -> Result<Response, Error> {
    client
        .get(url.replace("{api-key}", &env::var("BOARDROOM_API_KEY").unwrap()))
        .send()
        .await
}
