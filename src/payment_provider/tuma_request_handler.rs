use diesel::{r2d2, PgConnection};
use diesel::r2d2::{ConnectionManager};
use crate::chains::TumaSupportedChains;
use anyhow::{Result, anyhow};
use crate::chains::aptos::{SendFungibleTokenArgs, SendTokenTransactionArgs, WalletTransaction};
use crate::controller::currency_controller::Currency;
use crate::payment_provider::sender::{FiatSender, SendFiatACH, SendFiatMobile, SendFiatRequest};

pub struct MobileFiatRequest {
    pub number: String,
    pub currency: Currency,
    pub amount: f64,
    pub network_id: String
}

pub struct ACHFiatRequest {
    pub account: String,
    pub bank_id: String,
    pub amount: f64,
    pub currency: Currency
}

pub struct CryptoRequest {
   pub chain: TumaSupportedChains,
    pub to: String,
    pub token: Currency,
    pub amount: f64,
    pub on_ramp_request_id: String
}


pub enum TumaRequest {
    MobileFiat(MobileFiatRequest),
    ACHFiat(ACHFiatRequest),
    Crypto(CryptoRequest)
}

pub struct TumaRequestHandler {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    fiat_sender: FiatSender
}


impl TumaRequestHandler {

    pub fn new(pool: r2d2::Pool<ConnectionManager<PgConnection>>, fiat_sender: FiatSender) -> Self {
        Self {
            pool,
            fiat_sender
        }
    }


    pub async fn send(&mut self, req: TumaRequest)->Result<String>{

        match req {
            TumaRequest::MobileFiat(payload)=> {
                self.fiat_sender.send(SendFiatRequest::MOBILE(SendFiatMobile {
                    currency: payload.currency,
                    amount: payload.amount,
                    network_id: payload.network_id,
                    phone: payload.number
                })).await
            },
            TumaRequest::ACHFiat(payload)=>{
                self.fiat_sender.send(SendFiatRequest::BANK(SendFiatACH {
                    amount: payload.amount,
                    currency: payload.currency,
                    bank_id: payload.bank_id,
                    account_number: payload.account
                })).await
            }
            TumaRequest::Crypto(payload)=>{
                match payload.chain {
                    TumaSupportedChains::APTOS(mut wallet )=>{
                        let token_address = match payload.token.address {
                            Some(a)=>a,
                            None=>return Err(anyhow!("token_address_not_found"))
                        };
                        let scale = match payload.token.decimals {
                            Some(v) => 10**v,
                            None=>return Err(anyhow!("tokens_should_have_a_scale"))
                        };
                        match &payload.token.is_fungible_asset {
                            Some(true) =>{
                                wallet.send(WalletTransaction::SendFungibleToken(SendFungibleTokenArgs {
                                    on_ramp_request_id: payload.on_ramp_request_id,
                                    amount: payload.amount.to_string(),
                                    token: token_address,
                                    scale,
                                    to_account: payload.to
                                }))
                            },
                            _=>{
                                wallet.send(WalletTransaction::SendToken(SendTokenTransactionArgs {
                                    on_ramp_request_id: payload.on_ramp_request_id,
                                    amount: payload.amount.to_string(),
                                    token_type: Some(token_address),
                                    scale,
                                    to_account: payload.to
                                }))
                            }
                        }
                    }
                }
            }
        }

    }


}