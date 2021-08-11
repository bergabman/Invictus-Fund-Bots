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

pub async fn api_general() -> Result<ApiFundsGeneral> {
    let api_response = reqwest::get("https://api.invictuscapital.com/v2/funds")
        .await?
        .json::<ApiFundsGeneral>()
        .await?;
    Ok(api_response)
}

pub async fn fund_nav(fund_name: &str) -> Result<String> {
    let fund_to_check = normalize_fund_name(fund_name)?;
    let api_response = api_general().await?;
    let funds_general_raw = api_response.data;
    for fund in funds_general_raw {
        if fund.name == fund_to_check {
            let mut nav = fund.nav_per_token;
            nav.truncate(nav.find(".").unwrap() + 4);
            return Ok(nav)
        }
    }
    Ok("notfound".into())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FundPie {
    status: String,
    pub assets: Vec<FundPieAsset>
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FundPieAsset {
    pub ticker: String,
    pub name: String,
    pub value: String,
    pub amount: String,
    pub price: String,
    pub percentage: String
}

impl FundPie {
    pub async fn fund_pie(fund_name: &str) -> Result<FundPie> {
        let fund_to_check = normalize_fund_name(fund_name)?;
        let pie = reqwest::get(format!("https://api.invictuscapital.com/v2/funds/{}/pie", fund_to_check))
            .await?
            .json::<FundPie>()
            .await?;
        
        Ok(pie)
    }
    
    pub fn remove_small_assets(&mut self) {
        self.stablecoin_summary();
        self.assets.retain(|asset| (asset.percentage.parse::<f64>().unwrap()) > 1.0);
    }

    pub fn stablecoin_summary(&mut self) {
        let mut usd_percentage = 0.0;
        let mut usd_value = 0.0;
        for asset in self.assets.iter_mut() {
            match asset.ticker.as_ref() {
                "USD" | "BUSD" | "BUSD-T" => {
                    usd_percentage += asset.percentage.parse::<f64>().unwrap();
                    usd_value += asset.value.parse::<f64>().unwrap();
                    asset.percentage = "0.0".to_string();
                    asset.value = "0.0".to_string();
                }
                &_ => {}
            }
        }
        for asset in self.assets.iter_mut() {
            match asset.ticker.as_ref() {
                "USD" => {
                    asset.percentage = usd_percentage.to_string();
                    asset.percentage.truncate(asset.percentage.find(".").unwrap_or(1) + 3);
                    asset.value = usd_value.to_string();
                }
                &_ => {}
            }
        }
    }
}
#[derive(Debug, Deserialize, Serialize)]
pub struct FundNav {
    status: String,
    circulating_supply: String,
    net_asset_value: String,
    nav_per_token: String,
    name: String,
    ticker: String,
    price: String,
    pub assets: Vec<FundNavAsset>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FundNavAsset {
    pub name: String,
    pub ticker: String,
    pub usd_value: String
}

impl FundNav {
    pub async fn fund_nav(fund_name: &str) -> Result<FundNav> {
        let fund_to_check = normalize_fund_name(fund_name)?;
        let pie = reqwest::get(format!("https://api.invictuscapital.com/v2/funds/{}/nav", fund_to_check))
            .await?
            .json::<FundNav>()
            .await?;
        
        Ok(pie)
    }

    pub fn net_asset_value(&self) -> String {
        self.net_asset_value.clone()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FundPerf {
    status: String,
    pub percentage: String
}

pub async fn fund_perf(fund_name: &str, range: &str) -> Result<String> {
    let fund_to_check = normalize_fund_name(fund_name)?;
    let fund_performance = reqwest::get(format!("https://api.invictuscapital.com/v2/funds/{}/movement?range={}", fund_to_check, range))
        .await?
        .json::<FundPerf>()
        .await?;
    Ok(fund_performance.percentage)
}


pub fn normalize_fund_name(got_name: &str) -> Result<String> {
    match got_name {
        "c20" | "crypto20" | "C20" => Ok("crypto20".into()),
        "c10" | "crypto10" | "C10" => Ok("crypto10".into()),
        "iba" | "bitcoin-alpha" | "IBA" => Ok("bitcoin-alpha".into()),
        "ihf" | "hyperion" | "IHF" => Ok("hyperion".into()),
        "iml" | "margin-lending" | "IML" => Ok("margin-lending".into()),
        "igp" | "gold-plus" | "IGP" => Ok("gold-plus".into()),
        "ems" | "emerging-markets-solar" | "EMS" => Ok("emerging-markets-solar".into()),
        &_ => return Err(anyhow!("notfound"))
    }
}