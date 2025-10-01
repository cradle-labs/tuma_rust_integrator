use anyhow::{Result, anyhow};
use crate::controller::currency_controller::Currency;
use crate::pretium::{OffRampRequestMobile, PretiumProcessRequest, PretiumProcessResponse, PretiumService};

pub struct SendFiatMobile {
    pub amount: f64,
    pub phone: String,
    pub network_id: String,
    pub currency: Currency
}

pub struct SendFiatACH {
    pub amount: f64,
    pub account_number: String,
    pub bank_id: String,
    pub currency: Currency
}


pub enum SendFiatRequest {
    MOBILE(SendFiatMobile),
    BuyGoodsMobile(SendFiatMobile),
    BANK(SendFiatACH)
}

#[derive(Debug,Clone)]
pub struct FiatSender {
    pretium: PretiumService
}

impl FiatSender {
    pub fn new(pretium: PretiumService)->Self {
        Self {
            pretium
        }
    }

    pub async fn send(&mut self, req: SendFiatRequest) -> Result<String> {

        let process_request = match req {
            SendFiatRequest::MOBILE(d)=> PretiumProcessRequest::OffRampMobile(OffRampRequestMobile {
                amount: d.amount.to_string(),
                currency: d.currency.symbol,
                phone: d.phone,
                network: d.network_id
            }),
            SendFiatRequest::BuyGoodsMobile(d)=> PretiumProcessRequest::MakePaymentMobileBuyGoods(OffRampRequestMobile {
                amount: d.amount.to_string(),
                currency: d.currency.symbol,
                phone: d.phone,
                network: d.network_id
            }),
            _=>return Err(anyhow!("unsupported_off_ramp_engine"))
        };

        let res = self.pretium.process(process_request).await?;

        match res {
            PretiumProcessResponse::OffRampMobile(d)=>Ok(d.transaction_code),
            _=> Err(anyhow!("unable_to_resolve_provider_response"))
        }
    }
}

