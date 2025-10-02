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
                },
                Currency {
                    symbol: "USDC".to_string(),
                    name: "USDC".to_string(),
                    decimals: Some(6),
                    address: Some("0xbae207659db88bea0cbead6da0ed00aac12edcdda169e591cd41c94180b46f3b".to_string()),
                    chain: Some("aptos".to_string()),
                    id: "usdc-apt".to_string(),
                    description: "USDC on APTOS".to_string(),
                    country: None,
                    currency_type: CurrencyType::Crypto,
                    is_fungible_asset: Some(true)
                },
                Currency {
                    symbol: "USDt".to_string(),
                    name: "USDt".to_string(),
                    decimals: Some(6),
                    address: Some("0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b".to_string()),
                    chain: Some("aptos".to_string()),
                    id: "usdt-apt".to_string(),
                    description: "USDt on APTOS".to_string(),
                    country: None,
                    currency_type: CurrencyType::Crypto,
                    is_fungible_asset: Some(true)
                },
                Currency {
                    symbol: "GUI".to_string(),
                    name: "Gui Inu".to_string(),
                    decimals: Some(6),
                    address: Some("0x0009da434d9b873b5159e8eeed70202ad22dc075867a7793234fbc981b63e119".to_string()),
                    chain: Some("aptos".to_string()),
                    id: "gui-apt".to_string(),
                    description: "GUI".to_string(),
                    country: None,
                    currency_type: CurrencyType::Crypto,
                    is_fungible_asset: Some(true)
                },
                Currency {
                    symbol: "WBTC".to_string(),
                    name: "Wrapped BTC".to_string(),
                    decimals: Some(8),
                    address: Some("0x68844a0d7f2587e726ad0579f3d640865bb4162c08a4589eeda3f9689ec52a3d".to_string()),
                    chain: Some("aptos".to_string()),
                    id: "wbtc-apt".to_string(),
                    description: "WBTC".to_string(),
                    country: None,
                    currency_type: CurrencyType::Crypto,
                    is_fungible_asset: Some(true)
                },
                Currency {
                    symbol: "xBTC".to_string(),
                    name: "OKX Wrapped BTC".to_string(),
                    decimals: Some(8),
                    address: Some("0x81214a80d82035a190fcb76b6ff3c0145161c3a9f33d137f2bbaee4cfec8a387".to_string()),
                    chain: Some("aptos".to_string()),
                    id: "xbtc-apt".to_string(),
                    description: "xBTC".to_string(),
                    country: None,
                    currency_type: CurrencyType::Crypto,
                    is_fungible_asset: Some(true)
                },
                Currency {
                    symbol: "xBTC".to_string(),
                    name: "OKX Wrapped BTC".to_string(),
                    decimals: Some(8),
                    address: Some("0x81214a80d82035a190fcb76b6ff3c0145161c3a9f33d137f2bbaee4cfec8a387".to_string()),
                    chain: Some("aptos".to_string()),
                    id: "xbtc-apt".to_string(),
                    description: "xBTC".to_string(),
                    country: None,
                    currency_type: CurrencyType::Crypto,
                    is_fungible_asset: Some(true)
                },
            ]
        }
    }


    pub fn get_currency_by_id(&self, id: String)->Option<Currency>{
        match self.currencies.iter().find(|c|c.id == id) {
            Some(c)=>Some(c.clone()),
            None=>None
        }
    }

    pub fn get_currency_by_token(&self, token: String)->Option<Currency>{
        match self.currencies.iter().find(|c|match &c.address {Some(v)=>v.to_string() == token, None =>false}) {
            Some(c)=>Some(c.clone()),
            None=>None
        }
    }
}