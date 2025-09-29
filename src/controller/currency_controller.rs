use std::ops::{Div, Mul};
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use crate::controller::aptos_panora_provider::AptosPanoraProvider;
use crate::pretium::{ExchangeRateRequest, PretiumProcessRequest, PretiumProcessResponse, PretiumService};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum CurrencyType {
    Crypto,
    Fiat
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Currency {
    pub currency_type: CurrencyType,
    pub name: String,
    pub symbol: String,
    pub id: String,
    pub country: Option<String>,
    pub description: String,
    pub chain: Option<String>,
    pub address: Option<String>,
    pub is_fungible_asset: Option<bool>,
    pub decimals: Option<u64>
}

impl Currency {

    pub async fn get_usd_exchange_rate(&self,
                                       panora: &mut AptosPanoraProvider, // TODO: should probably have a generic provider for different dexes on different chains
                                       pret: &mut PretiumService)->Result<f64> {

        match &self.currency_type {
            CurrencyType::Fiat=>{

                let rate = pret.process(PretiumProcessRequest::ExchangeRate(ExchangeRateRequest {
                    currency: self.symbol.clone()
                })).await?;

                return match rate {
                    PretiumProcessResponse::ExchangeRate(d)=>Ok(d.quoted_rate),
                    _=> Err(anyhow!("exchange_rate_error"))
                }

            },
            CurrencyType::Crypto=>{
                let chain = match &self.chain {
                    Some(c)=>c.clone(),
                    None=>return Err(anyhow!("chain_not_found"))
                };

                match chain.as_str() {
                    "aptos"=>{

                        let token_address = match &self.address {
                            Some(a)=>a,
                            None=>return Err(anyhow!("token_not_specified"))
                        };

                        let usd_price = panora.get_usd_price(token_address.as_str()).await?;
                        return Ok(usd_price)
                    },
                    _=>{
                        return Err(anyhow!("chain_not_yet_supported"))
                    }
                };
            }
        }
    }

    pub async fn convert(panora_provider: &mut AptosPanoraProvider, pretium_service: &mut PretiumService, currency_a: Currency, currency_b: Currency, currency_a_amount: f64) ->Result<f64> {
        let currency_a_in_usd = currency_a.get_usd_exchange_rate(panora_provider, pretium_service).await?;
        let currency_b_in_usd = currency_b.get_usd_exchange_rate(panora_provider, pretium_service).await?;

        let a_in_usd = currency_a_amount.div(currency_a_in_usd);
        let usd_in_b = a_in_usd.mul(currency_b_in_usd);

        Ok(usd_in_b)
    }

}