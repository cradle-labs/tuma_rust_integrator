use std::collections::HashMap;
use std::env;
use reqwest::Client;
use anyhow::Result;
use serde::{Deserialize, Serialize};

const ON_RAMP_CALLBACK_ENDPOINT: &str = "";
const OFF_RAMP_CALLBACK_ENDPOINT: &str = "";

#[derive(Debug,Clone)]
pub struct PretiumService {
    client: Client,
    api_key: String,
    callback_on_ramp: String,
    callback_off_ramp: String
}

#[derive(Deserialize,Clone)]
pub struct PretiumResponseWrapper<T> {
    pub code: u64,
    pub message: String,
    pub data: T
}

#[derive(Deserialize,Serialize, Clone)]
pub struct ExchangeRateRequest {
    pub currency: String
}

#[derive(Deserialize,Serialize,Clone)]
pub struct ExchangeRateResponse {
    pub buying_rate: f64,
    pub selling_rate: f64,
    pub quoted_rate: f64
}


#[derive(Deserialize, Serialize, Clone)]
pub struct OnRampRequestMobileReq {
    pub phone: String,
    pub amount: String,
    pub network: String,
    pub currency_id: String
}

#[derive(Deserialize,Serialize, Clone)]
pub struct OnRampRequestMobileResponse {
    pub transaction_code: String,
    pub status: String,
    pub message: String
}

#[derive(Deserialize,Serialize, Clone)]
pub struct OffRampRequestMobile {
    pub phone: String,
    pub amount: String,
    pub network: String,
    pub currency: String
}

#[derive(Deserialize,Serialize,Clone)]
pub struct OffRampMobileResponse {
    pub transaction_code: String,
    pub status: String,
    pub message: String,
    pub receipt_number: Option<String>
}

pub enum PretiumProcessRequest {
    ExchangeRate(ExchangeRateRequest),
    OnRampMobile(OnRampRequestMobileReq),
    OffRampMobile(OffRampRequestMobile)
}

pub enum PretiumProcessResponse {
    ExchangeRate(ExchangeRateResponse),
    OnRampMobile(OnRampRequestMobileResponse),
    OffRampMobile(OffRampMobileResponse)
}


impl PretiumService {
    pub fn new(api_key: String)->Result<Self> {
        let client = Client::new();
        let on_ramp = env::var("ON_RAMP_CALLBACK_ENDPOINT")?;
        let off_ramp = env::var("OFF_RAMP_CALLBACK_ENDPOINT")?;
        let callback_on_ramp = format!("{}", on_ramp);
        let callback_off_ramp = format!("{}", off_ramp);

        Ok(Self {
            client,
            api_key,
            callback_on_ramp,
            callback_off_ramp
        })
    }

    fn to_payload<'a>(&'a self, req: &'a PretiumProcessRequest) ->HashMap<&'a str, &'a str> {

        let mut payload = HashMap::new();


        match req {
            PretiumProcessRequest::ExchangeRate(data)=>{
                payload.insert("currency_code", data.currency.as_str());
            },
            PretiumProcessRequest::OnRampMobile(data)=>{
                payload.insert("shortcode", data.phone.as_str());
                payload.insert("amount", data.amount.as_str());
                payload.insert("mobile_network", data.network.as_str());
                payload.insert("callback_url", self.callback_on_ramp.as_str());
            },
            PretiumProcessRequest::OffRampMobile(data)=>{
                payload.insert("shortcode", data.phone.as_str());
                payload.insert("amount", data.amount.as_str());
                payload.insert("type", "MOBILE");
                payload.insert("mobile_network", data.network.as_str());
                payload.insert("callback_url", self.callback_off_ramp.as_str());
            }
        }


        payload
    }

    fn to_path(&self, req: &PretiumProcessRequest)->String {

        match req {
            PretiumProcessRequest::ExchangeRate(_)=> "/exchange-rate".to_string(),
            PretiumProcessRequest::OnRampMobile(d)=>format!("/{}/collect", d.currency_id),
            PretiumProcessRequest::OffRampMobile(d)=>format!("/{}/disburse", d.currency)
        }
    }

    pub async fn process(&mut self, req: PretiumProcessRequest)->Result<PretiumProcessResponse> {

        let path = self.to_path(&req);
        let payload = self.to_payload(&req);

        let resp = self.client.post(path.as_str())
            .header("x-api-key", self.api_key.as_str())
            .json(&payload)
            .send()
            .await?;

        match req {
            PretiumProcessRequest::ExchangeRate(_)=>{
                let res = resp.json::<PretiumResponseWrapper<ExchangeRateResponse>>().await?;
                Ok(PretiumProcessResponse::ExchangeRate(res.data))
            },
            PretiumProcessRequest::OnRampMobile(_)=>{
                let res = resp.json::<PretiumResponseWrapper<OnRampRequestMobileResponse>>().await?;
                Ok(PretiumProcessResponse::OnRampMobile(res.data))
            },
            PretiumProcessRequest::OffRampMobile(_)=>{
                let res = resp.json::<PretiumResponseWrapper<OffRampMobileResponse>>().await?;
                Ok(PretiumProcessResponse::OffRampMobile(res.data))
            }
        }


    }
}