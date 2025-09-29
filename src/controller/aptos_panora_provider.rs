use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer};
use anyhow::Result;
use reqwest::Client;

fn de_f64<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    let v = serde_json::Value::deserialize(d)?;
    match v {
        serde_json::Value::Number(n) => n.as_f64().ok_or_else(|| de::Error::custom("invalid f64")),
        serde_json::Value::String(s) => s.parse::<f64>().map_err(|e| de::Error::custom(format!("invalid f64: {e}"))),
        _ => Err(de::Error::custom("invalid type for f64")),
    }
}

fn de_u64<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    let v = serde_json::Value::deserialize(d)?;
    match v {
        serde_json::Value::Number(n) => n.as_u64().ok_or_else(|| de::Error::custom("invalid u64")),
        serde_json::Value::String(s) => s.parse::<u64>().map_err(|e| de::Error::custom(format!("invalid u64: {e}"))),
        _ => Err(de::Error::custom("invalid type for u64")),
    }
}

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
    #[serde(rename = "chainId", deserialize_with = "de_u64")]
    pub chain_id: u64,
    #[serde(rename = "panoraId")]
    pub panora_id: String,
    #[serde(rename = "tokenAddress")]
    pub token_address: Option<String>,
    #[serde(rename = "faAddress")]
    pub fa_address: Option<String>,
    pub name: String,
    pub symbol: String,
    #[serde(deserialize_with = "de_f64")]
    pub decimals: f64,
    #[serde(rename = "usdPrice")]
    pub usd_price: String,
    #[serde(rename = "nativePrice")]
    pub native_price: String,
    #[serde(rename = "priceChange24H")]
    pub price_change_24h: String,
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
            .send().await?
            .error_for_status()?;

        let body = resp.json::<Vec<GetAssetUSDPriceResponse>>().await?;

        let chosen = body.first();

        let value = chosen.unwrap().usd_price.parse::<f64>()?;


        Ok(value)
    }
}