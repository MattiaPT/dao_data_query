use std::{env, error::Error as err, fs, io::prelude::*};

use log::info;
use reqwest::Client;

pub mod utils;
use utils::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn err>> {
    env_logger::init();

    let client = Client::new();

    info!("Initializing Query");

    let response = &client
        .get(CNAMES_URL.replace("{api-key}", &env::var("BOARDROOM_API_KEY")?))
        .send()
        .await?
        .text()
        .await?;

    let deserialized: ProtocolResponse = serde_json::from_str(&response)?;

    let mut file = fs::File::create(&RAW_PROTOCOLS)?;
    file.write_all(serde_json::to_string_pretty(&deserialized)?.as_bytes())?;
    info!("Wrote raw data to {}", &RAW_PROTOCOLS);

    let cname_csv_data = "dao,cname\n".to_string()
        + &deserialized
            .data
            .iter()
            .map(|protocol| format!("{},{}", protocol.name, protocol.cname))
            .collect::<Vec<String>>()
            .join("\n");

    let mut file = fs::File::create(&CNAMES)?;
    file.write_all(cname_csv_data.as_bytes())?;
    info!("Wrote cname list to {}", CNAMES);

    Ok(())
}
