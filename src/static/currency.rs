use crate::controller::currency_controller::{Currency, CurrencyType};

pub struct CurrencyStaticData {
    pub currencies: Vec<Currency>
}


impl CurrencyStaticData {
    pub fn new()-> Self {
        Self {
            currencies: vec![
                Currency {
                    symbol: "KES".to_string(),
                    name: "Kenyan Shilling".to_string(),
                    decimals: None,
                    address: None,
                    chain: None,
                    is_fungible_asset: None,
                    currency_type: CurrencyType::Fiat,
                    id: "kes".to_string(),
                    description: "Currency of the Republic of Kenya".to_string(),
                    country: Some("Kenya".to_string())
                },
                Currency {
                    symbol: "APT".to_string(),
                    name: "Aptos Coin".to_string(),
                    decimals: Some(8),
                    address: Some("0xa".to_string()),
                    chain: Some("aptos".to_string()),
                    id: "apt".to_string(),
                    description: "Native currency on Aptos".to_string(),
                    country: None,
                    currency_type: CurrencyType::Crypto,
                    is_fungible_asset: Some(true)
                }
            ]
        }
    }


    pub fn get_currency_by_id(&self, id: String)->Option<Currency>{
        match self.currencies.iter().find(|c|c.id == id) {
            Some(c)=>Some(c.clone()),
            None=>None
        }
    }
}