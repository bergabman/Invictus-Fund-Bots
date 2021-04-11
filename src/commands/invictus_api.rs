use serde_derive::{Serialize, Deserialize};
use anyhow::{Result, anyhow};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiFundsGeneral {
    pub status: String,
    pub data: Vec<FundGeneral>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FundGeneral {
    pub circulating_supply: String,
    pub net_asset_value: String,
    pub nav_per_token: String,
    pub name: String,
    pub ticker: String
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Asset {
    name: String,
    ticker: String,
    usd_value: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Crypto10 {
    status: String,
    circulating_supply: String,
    net_asset_value: String,
    nav_per_token: String,
    name: String,
    ticker: String,
    price: String,
    assets: Vec<Asset>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FundMovement {
    status: String,
    percentage: String
}

pub async fn api_general() -> Result<ApiFundsGeneral> {
    let api_response = reqwest::get("https://api.invictuscapital.com/v2/funds")
        .await?
        .json::<ApiFundsGeneral>()
        .await?;
    // println!("{:?}", api_response);

    // let funds: funds = api_response;
    Ok(api_response)
}

pub async fn api_c10_full() -> Result<Crypto10> {
    let api_response = reqwest::get("https://api.invictuscapital.com/v2/funds/crypto10/nav")
        .await?
        .json::<Crypto10>()
        .await?;
    // println!("{:?}", api_response);

    // let funds: funds = api_response;
    Ok(api_response)
}

pub async fn api_c10_mov() -> Result<String> {
    let time_ranges = vec!["1", "12", "24"];
    let mut movements = String::new();

    for range in time_ranges {
        let fund_movement = reqwest::get(format!("https://api.invictuscapital.com/v2/funds/crypto10/movement?range={}h", range))
        .await?
        .json::<FundMovement>()
        .await?;
        movements.push_str(&format!("| {}h {}% ", range, fund_movement.percentage));
    }
    
    Ok(movements)
}

pub async fn api_c10_mov_time(timeframe: String) -> Result<String> {
    let fund_movement = reqwest::get(format!("https://api.invictuscapital.com/v2/funds/crypto10/movement?range={}h", timeframe))
        .await?
        .json::<FundMovement>()
        .await?;
    Ok(fund_movement.percentage)
}