use std::collections::HashMap;
use std::env;
use reqwest::{Client, Url};
use anyhow::Result;
use serde::{Deserialize, Serialize};
// --- lenient deserializers for providers that send numbers as strings ---
use serde::de::{self, Deserializer};

fn de_f64<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    let v = serde_json::Value::deserialize(d)?;
    match v {
        serde_json::Value::Number(n) => n.as_f64().ok_or_else(|| de::Error::custom("invalid f64")),
        serde_json::Value::String(s) => s.parse::<f64>().map_err(|e| de::Error::custom(format!("invalid f64: {e}"))),
        _ => Err(de::Error::custom("invalid type for f64")),
    }
}

fn de_u16<'de, D: Deserializer<'de>>(d: D) -> Result<u16, D::Error> {
    let v = serde_json::Value::deserialize(d)?;
    match v {
        serde_json::Value::Number(n) => n.as_u64().map(|x| x as u16).ok_or_else(|| de::Error::custom("invalid u16")),
        serde_json::Value::String(s) => s.parse::<u16>().map_err(|e| de::Error::custom(format!("invalid u16: {e}"))),
        _ => Err(de::Error::custom("invalid type for u16")),
    }
}

const ON_RAMP_CALLBACK_ENDPOINT: &str = "";
const OFF_RAMP_CALLBACK_ENDPOINT: &str = "";

#[derive(Debug,Clone)]
pub struct PretiumService {
    client: Client,
    api_key: String,
    callback_on_ramp: String,
    callback_off_ramp: String,
    callback_buy_goods_off_ramp: String
}

#[derive(Deserialize,Serialize,Clone)]
pub struct PretiumResponseWrapper<T> {
    #[serde(deserialize_with = "de_u16")] pub code: u16,
    pub message: String,
    pub data: T
}

#[derive(Deserialize,Serialize, Clone)]
pub struct ExchangeRateRequest {
    pub currency: String
}

#[derive(Deserialize,Serialize,Clone)]
pub struct ExchangeRateResponse {
    #[serde(deserialize_with = "de_f64")]
    pub buying_rate: f64,
    #[serde(deserialize_with = "de_f64")]
    pub selling_rate: f64,
    #[serde(deserialize_with = "de_f64")]
    pub quoted_rate: f64,
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
    pub currency: String,
    pub is_buy_goods: Option<bool>
}

#[derive(Deserialize,Serialize,Clone)]
pub struct PayBillRequestMobile {
    pub pay_bill: String,
    pub account_number: String,
    pub network: String,
    pub currency: String,
    pub amount: String
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
    OffRampMobile(OffRampRequestMobile),
    MakePaymentMobileBuyGoods(OffRampRequestMobile),
    PayBillMobile(PayBillRequestMobile)
}

pub enum PretiumProcessResponse {
    ExchangeRate(ExchangeRateResponse),
    OnRampMobile(OnRampRequestMobileResponse),
    OffRampMobile(OffRampMobileResponse),
    MakePaymentMobileBuyGoods(OffRampRequestMobile),
    PayBillMobile(OffRampRequestMobile)
}


impl PretiumService {
    pub fn new(api_key: String)->Result<Self> {
        let client = Client::new();
        let on_ramp = env::var("ON_RAMP_CALLBACK_ENDPOINT")?;
        let off_ramp = env::var("OFF_RAMP_CALLBACK_ENDPOINT")?;
        let buy_goods_off_ramp = env::var("CALLBACK_BUY_GOODS_OFF_RAMP")?;
        let callback_on_ramp = format!("{}", on_ramp);
        let callback_off_ramp = format!("{}", off_ramp);

        Ok(Self {
            client,
            api_key,
            callback_on_ramp,
            callback_off_ramp,
            callback_buy_goods_off_ramp: buy_goods_off_ramp
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
            },
            PretiumProcessRequest::MakePaymentMobileBuyGoods(data)=>{
                let transaction_type = match data.is_buy_goods {
                    Some(v)=> if v { "BUY_GOODS" } else { "MOBILE" },
                    None=> "MOBILE"
                };
                payload.insert("shortcode", data.phone.as_str());
                payload.insert("amount", data.amount.as_str());
                payload.insert("type", transaction_type);
                payload.insert("mobile_network", data.network.as_str());
                payload.insert("callback_url", self.callback_buy_goods_off_ramp.as_str());
            },
            PretiumProcessRequest::PayBillMobile(data)=>{
                payload.insert("shortcode", data.pay_bill.as_str());
                payload.insert("amount", data.amount.as_str());
                payload.insert("account_number", data.account_number.as_str());
                payload.insert("type", "PAYBILL");
                payload.insert("mobile_network", data.network.as_str());
                payload.insert("callback_url", self.callback_buy_goods_off_ramp.as_str());
            }
        }


        payload
    }

    fn to_path(&self, req: &PretiumProcessRequest)->String {

        match req {
            PretiumProcessRequest::ExchangeRate(_)=> "/v1/exchange-rate".to_string(),
            PretiumProcessRequest::OnRampMobile(d)=>format!("/{}/collect", d.currency_id.to_lowercase()),
            PretiumProcessRequest::OffRampMobile(d)=>format!("/{}/disburse", d.currency.to_lowercase()),
            PretiumProcessRequest::MakePaymentMobileBuyGoods(d)=>format!("/{}/disburse", d.currency.to_lowercase()),
            PretiumProcessRequest::PayBillMobile(d)=>format!("/{}/disburse", d.currency.to_lowercase()),
        }
    }

    pub async fn process(&mut self, req: PretiumProcessRequest)->Result<PretiumProcessResponse> {
        let client = self.client.clone();
        let path = self.to_path(&req);
        let payload = self.to_payload(&req);

        println!("Payload {:?}", payload);
        println!("API KEY {:?}", self.api_key);
        println!("Path {:?}", path);

        let base = Url::parse("https://api.xwift.africa/").expect("valid base url");
        let url = base.join(path.trim_start_matches('/')).expect("valid joined url");

        let resp = match client.post(url)
            .header("x-api-key", self.api_key.as_str())
            .json(&payload)
            .send()
            .await {
            Ok(res)=>res,
            Err(e)=>{
                println!("Something went wrong building the client {}",e);
                return Err(anyhow::anyhow!("unable_to_build_client::{}",e))
            }
        };
        // convert non-2xx into errors so we don't try to JSON-decode error HTML/text
        let resp = resp.error_for_status()?;
        // read body once for robust logging + flexible decode (ignores bad content-types)
        let body = resp.bytes().await?;
        eprintln!("RAW BODY: {}", String::from_utf8_lossy(&body));
        println!("able to build");

        match req {
            PretiumProcessRequest::ExchangeRate(_)=>{
                let res: PretiumResponseWrapper<ExchangeRateResponse> = serde_json::from_slice(&body)?;
                Ok(PretiumProcessResponse::ExchangeRate(res.data))
            },
            PretiumProcessRequest::OnRampMobile(_)=>{
                let res: PretiumResponseWrapper<OnRampRequestMobileResponse> = serde_json::from_slice(&body)?;
                Ok(PretiumProcessResponse::OnRampMobile(res.data))
            },
            PretiumProcessRequest::OffRampMobile(_)=>{
                let res: PretiumResponseWrapper<OffRampMobileResponse> = serde_json::from_slice(&body)?;
                Ok(PretiumProcessResponse::OffRampMobile(res.data))
            },
            PretiumProcessRequest::MakePaymentMobileBuyGoods(_)=>{
                let res: PretiumResponseWrapper<OffRampMobileResponse> = serde_json::from_slice(&body)?;
                Ok(PretiumProcessResponse::OffRampMobile(res.data))
            },
            PretiumProcessRequest::PayBillMobile(_)=>{
                let res: PretiumResponseWrapper<OffRampMobileResponse> = serde_json::from_slice(&body)?;
                Ok(PretiumProcessResponse::OffRampMobile(res.data))
            }
        }


    }
}