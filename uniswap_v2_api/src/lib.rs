use serde_derive::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use etherscan_io_api as etherscan;

pub const UNISWAP_V2: &str = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v2";

pub const ICAP:&str = "0xd83c569268930fadad4cde6d0cb64450fef32b65";

pub async fn uniswap_icap() -> Result<String> {
    let uniswap_url = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v2";
    let uniswap_json: serde_json::Value = reqwest::Client::new()
        .post(uniswap_url)
        .json(&serde_json::json!({
            "operationName":"NavPrice",
            "variables":{"token_address":"0xd83c569268930fadad4cde6d0cb64450fef32b65","dai_token_address":"0x6b175474e89094c44da98b954eedeac495271d0f"},
                "query":"query NavPrice($token_address: String!, $dai_token_address: String!) {\n  pairs(where: {token0: $dai_token_address, token1: $token_address}) {\n    token0Price\n    __typename\n  }\n}\n"
                }))
        .send()
        .await?
        .json()
        .await?;
    // info!("{}", uniswap_json["data"]["pairs"][0]["token0Price"]);
    let mut icap_price = format!("{}",uniswap_json["data"]["pairs"][0]["token0Price"]);
    icap_price = icap_price.replace("\"", "");

    Ok(icap_price)
}

pub async fn token_price_at_block(token: &str, block: i64) -> Result<f64> {
    
    let jsonconcat = &serde_json::json!({
        "operationName":"TokenPrice",
        "variables":{"id": token, "block": block},
        "query":"query TokenPrice($id: String!, $block: Int!) {\n  token(id: $id, block: {number: $block}){\n    derivedETH\n}\n}"});
    let uniswap_json: serde_json::Value = reqwest::Client::new()
        .post(UNISWAP_V2)
        // .json(&serde_json::json!({"operationName":"IcapPrice","variables":{"id":"0xd83c569268930fadad4cde6d0cb64450fef32b65"},"query":"query IcapPrice($id: String!) {\n  token(id: $id){\n    derivedETH\n}\n}"}))
        // .json(&serde_json::json!({
        //                         "operationName":"IcapPrice",
        //                         "variables":{"id":"0xd83c569268930fadad4cde6d0cb64450fef32b65", "block": block_num },
        //                         "query":"query IcapPrice($id: String!, $block: Int!) {\n  token(id: $id, block: {number: $block}){\n    symbol\n    derivedETH\n}\n}"}))
        .json(jsonconcat)
        .send()
        .await?
        .json()
        .await?;

    // println!("----{:?}", jsonconcat);
    // println!("----{:?}", uniswap_json);
    let mut token_price = format!("{}",uniswap_json["data"]["token"]["derivedETH"]);
    // let mut icap_price = format!("{}",uniswap_json);
    token_price = token_price.replace("\"", "");

    // println!("---token price{:?}", token_price);
    Ok(token_price.parse::<f64>()?)
}


pub async fn eth_price_at_block(block: i64) -> Result<f64> {
    
    let jsonconcat = &serde_json::json!({
        "operationName":"EthPrice",
        "variables":{"block": block},
        "query":"query EthPrice($block: Int!) {\n  bundle(id: 1, block: {number: $block}){\nethPrice\n}\n}"});
    let uniswap_json: serde_json::Value = reqwest::Client::new()
        .post(UNISWAP_V2)
        .json(jsonconcat)
        .send()
        .await?
        .json()
        .await?;

    // println!("{:?}", uniswap_json);
    let mut eth_price = format!("{}",uniswap_json["data"]["bundle"]["ethPrice"]);
    // let mut icap_price = format!("{}",uniswap_json);
    eth_price = eth_price.replace("\"", "");
    // println!("----eth price {}", eth_price);

    Ok(eth_price.parse::<f64>()?)
}

pub async fn fund_perf(fund_ticker: &str, playing: &str) -> Result<String> {
    // println!("fund ticker {}", fund_ticker);
    if fund_ticker.to_lowercase() != ICAP {
        return Err(anyhow!("Unknown fund"))
    }

    let epoch = etherscan::Epoch::now();
    let time_range = playing_to_secs(playing)?;

    let last_block = etherscan::get_last_block_num().await?;
    let previous_block = etherscan::get_block_by_timestamp(&format!("{}", epoch - time_range)).await?;

    let eth_price_now = eth_price_at_block(last_block - 10).await?;
    let eth_price_previous = eth_price_at_block(previous_block).await?;

    let api_response = token_price_at_block(ICAP, last_block - 10).await?;
    let api_response_2 = match token_price_at_block(ICAP, previous_block).await {
        Ok(response) => response,
        Err(_) => return Ok("0.0".to_string()),
    };
    let token_price_now = api_response * eth_price_now;
    let token_price_previous = api_response_2 * eth_price_previous;
    // println!("token price now {}\n{} token price prev {}", token_price_now,playing, token_price_previous );
    let mut percentage = (token_price_now / ( token_price_previous / 100.0 ) - 100.0).to_string();
    percentage.truncate(percentage.find(".").unwrap_or(1) + 3 );
    Ok(percentage)
}

pub async fn fund_nav(fund_ticker: &str) -> Result<String> {
    if fund_ticker.to_lowercase() != "icap" {
        return Err(anyhow!("Unknown fund"))
    }
    let last_block = etherscan::get_last_block_num().await?;
    let eth_price_now = eth_price_at_block(last_block - 10).await?;
    let api_response = token_price_at_block(ICAP, last_block - 10).await?;
    let token_price_now = api_response * eth_price_now;
    Ok(token_price_now.to_string())
}

fn playing_to_secs(playing: &str) -> Result<u64> {
    let mut numbers = playing.to_string();
    numbers.truncate(numbers.len() - 1);
    let seconds:u64 = numbers.parse().unwrap();
    match playing.chars().last().unwrap().to_lowercase().to_string().as_ref() {
        "w" => return Ok(seconds*7*24*60*60),
        "d" => return Ok(seconds*24*60*60),
        "h" => return Ok(seconds*60*60),
        _ => return Err(anyhow!("failed to parse time range")),
    }
}


// pub fn eth_block_back(block: i64) -> i64 {

// }

