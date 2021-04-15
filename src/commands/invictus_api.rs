use serde_derive::{Serialize, Deserialize};
use anyhow::Result;

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
    pub name: String,
    pub ticker: String,
    pub usd_value: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Crypto10Pie {
    status: String,
    pub assets: Vec<PieAsset>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PieAsset {
    pub ticker: String,
    pub name: String,
    pub value: String,
    pub amount: String,
    pub price: String,
    pub percentage: String
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
    pub assets: Vec<Asset>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FundMovement {
    status: String,
    percentage: String
}

impl Crypto10Pie {
    /// remove assets from list that are smaller than 0.1% of the fund. Mostly DASH investments and assets with 0 value
    pub fn remove_small_assets(&mut self) {
        self.assets.retain(|asset| (asset.percentage.parse::<f64>().unwrap()) > 1.0 || asset.ticker == "USD");
    }

    pub fn get_cash_allocation_percent(&self) -> Option<f64> {
        for asset in &self.assets {
            if asset.ticker == "USD" {
                return Some(asset.percentage.parse::<f64>().unwrap())
            }
        }
        return None;
    }
}

impl Crypto10 {
    pub fn net_fund_value(&self) -> String {
        self.net_asset_value.clone()
    }
}

pub async fn api_general() -> Result<ApiFundsGeneral> {
    let api_response = reqwest::get("https://api.invictuscapital.com/v2/funds")
        .await?
        .json::<ApiFundsGeneral>()
        .await?;
    Ok(api_response)
}

pub async fn api_c10_full() -> Result<Crypto10> {
    let api_response = reqwest::get("https://api.invictuscapital.com/v2/funds/crypto10/nav")
        .await?
        .json::<Crypto10>()
        .await?;
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
        movements.push_str(&format!("**{}h** {}%\n", range, fund_movement.percentage));
    }
    
    Ok(movements)
}

pub async fn api_c10_pie() -> Result<Crypto10Pie> {

    let pie = reqwest::get("https://api.invictuscapital.com/v2/funds/crypto10/pie")
        .await?
        .json::<Crypto10Pie>()
        .await?;
    
    Ok(pie)
}

pub async fn api_c10_mov_time(timeframe: String) -> Result<String> {
    let fund_movement = reqwest::get(format!("https://api.invictuscapital.com/v2/funds/crypto10/movement?range={}h", timeframe))
        .await?
        .json::<FundMovement>()
        .await?;
    Ok(fund_movement.percentage)
}