use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use reqwest::blocking::Client;

const BASE_API: &str = "https://api.panora.exchange";

#[derive(Debug, Clone, Deserialize)]
pub struct AptosPanoraProvider {
    api_key: String
}

#[derive(Deserialize,Serialize,Clone, Debug)]
pub struct GetAssetUSDPriceRequest {
    pub token_address: String
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetAssetUSDPriceResponse {
    pub chainId: String,
    pub panoraId: String,
    pub tokenAddress: Option<String>,
    pub faAddress: Option<String>,
    pub name: String,
    pub symbol: String,
    pub decimals: u64,
    pub usdPrice: String,
    pub nativePrice: String,
    pub priceChange24H: String
}


impl AptosPanoraProvider {

    pub fn new()->Self {
        Self {
            api_key: String::from("a4^KV_EaTf4MW#ZdvgGKX#HUD^3IFEAOV_kzpIE^3BQGA8pDnrkT7JcIy#HNlLGi")
        }
    }


    pub async fn get_usd_price(&self, token_address: &str)->Result<f64>{

        let client = Client::new();

        let mut query_map = HashMap::new();
        query_map.insert("tokenAddress", token_address);


        let resp = client.get(format!("{BASE_API}/prices"))
            .header("x-api-key", self.api_key.as_str())
            .query(&query_map)
            .send()?;

        let data = resp.json::<GetAssetUSDPriceResponse>()?;

        let parsed_value = data.usdPrice.parse::<f64>()?;


        Ok(parsed_value)
    }
}