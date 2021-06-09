use serde_derive::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use serde_json::Value;
pub mod epoch;
pub use epoch::Epoch;

pub const ICAP: &str = "0xd83c569268930fadad4cde6d0cb64450fef32b65";

pub async fn get_last_block_num() -> Result<i64> {
    let epoch = Epoch::now();
    let last_block = get_block_by_timestamp(&epoch.to_string()).await?;
    Ok(last_block)

}

pub async fn get_block_by_timestamp(epoch: &str) -> Result<i64> {
    let block: Value = reqwest::get(format!("https://api.etherscan.io/api?module=block&action=getblocknobytime&timestamp={}&closest=before&apikey=NJ31CXQPAFU7BQRDQSFM97YSCQQRYXQTBC", epoch))
        .await?
        .json()
        .await?;
    let last_block = format!("{}", block["result"]).replace('"', "");
    Ok(last_block.parse::<i64>()?)
}

pub async fn eth_price() -> Result<f64> {
    let response: Value = reqwest::get(format!("https://api.etherscan.io/api?module=stats&action=ethprice&apikey=NJ31CXQPAFU7BQRDQSFM97YSCQQRYXQTBC"))
        .await?
        .json()
        .await?;
    println!("{:#?}", response);
    let eth_price = format!("{}", response["result"]["ethusd"]).replace('"', "");
    Ok(eth_price.parse::<f64>()?)
}
