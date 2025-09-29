use diesel::{r2d2, PgConnection};
use diesel::r2d2::{ConnectionManager};
use anyhow::{Result,anyhow};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::accounts::manager::PaymentMethodType;
use crate::chains::aptos::AptosWallet;
use crate::chains::TumaSupportedChains;
use crate::controller::aptos_panora_provider::AptosPanoraProvider;
use crate::controller::currency_controller::Currency;
use crate::schema::payment_method as PaymentMethodTable;
use crate::schema::on_ramp_requests as OnRampRequestsTable;
use crate::payment_provider::provider::{FiatPaymentProvider, PaymentProviderType};
use crate::payment_provider::tuma_request_handler::{CryptoRequest, TumaRequest, TumaRequestHandler};
use crate::pretium::{OnRampRequestMobileReq, PretiumProcessRequest, PretiumProcessResponse, PretiumService};
use crate::r#static::currency::CurrencyStaticData;
use crate::r#static::providers::StaticProviderData;

#[derive(Deserialize,Serialize)]
pub struct TransactionCallbackData {
    pub status: String,
    pub transaction_code: String,
    pub receipt_number: Option<String>,
    pub public_name: Option<String>,
    pub message: String
}

#[derive(Deserialize,Serialize, Queryable, Selectable)]
#[diesel(table_name = PaymentMethodTable)]
pub struct PaymentMethod {
    pub id: Uuid,
    pub owner: String,
    pub payment_method_type: Option<PaymentMethodType>,
    pub identity: String,
    pub provider_id: String,
    pub created_at: NaiveDateTime,
}

#[derive(Deserialize, Serialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::OnrampRequestStatus"]
pub enum OnRampRequestStatusEnum {
    Pending,
    Completed,
    Failed,
    Canceled
}

#[derive(Deserialize,Serialize,Insertable)]
#[diesel(table_name = OnRampRequestsTable)]
pub struct CreateOnRampRequest {
    pub requester: String,
    pub payment_method_id: Uuid,
    pub transaction_ref: Option<String>,
    pub data: Option<Value>,
    pub amount: BigDecimal,
    pub target_token: String
}


#[derive(Deserialize, Serialize, Queryable)]
#[diesel(table_name = OnRampRequestsTable)]
pub struct GetOnRampRequest {
    pub id: Uuid,
    pub requester: String,
    pub payment_method_id: Uuid,
    pub status: OnRampRequestStatusEnum,
    pub transaction_ref: Option<String>,
    pub data: Option<Value>,
    pub amount: Option<BigDecimal>,
    pub requested_at: NaiveDateTime,
    pub finalized_at: Option<NaiveDateTime>,
    pub target_token: String,
    pub final_token_quote: Option<BigDecimal>,
    pub on_chain_transaction_hash: Option<String>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OnRampRequest{
    pub payment_method_id: Uuid,
    pub amount: f64,
    pub target_token: String
}

pub struct OnRampHandler {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    pretium: PretiumService,
    panora: AptosPanoraProvider,
    providers: StaticProviderData,
    req_handler: TumaRequestHandler,
    currencies: CurrencyStaticData
}

impl OnRampHandler {

    pub fn new(pretium: PretiumService, panora: AptosPanoraProvider, pool: r2d2::Pool<ConnectionManager<PgConnection>>, req_handler: TumaRequestHandler)->Self {
        Self {
            pool,
            pretium,
            providers: StaticProviderData::new(),
            req_handler,
            currencies: CurrencyStaticData::new(),
            panora
        }
    }

    pub async fn get_payment_method(&mut self, payment_method_id: Uuid)->Result<PaymentMethod> {
        use crate::schema::payment_method::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_)=>return Err(anyhow!("failed_to_get_a_connection"))
        };

        let res = payment_method.filter(
            id.eq(payment_method_id)
        ).get_result::<PaymentMethod>(&mut conn)?;

        Ok(res)
    }

    pub async fn get_provider(&self, provider_id: String)->Result<FiatPaymentProvider> {
        match self.providers.get_id(provider_id.as_str()) {
            Some(d)=>Ok(d),
            None=>Err(anyhow!("provider_not_found"))
        }
    }


    pub async fn create_on_ramp_request(&mut self, req: OnRampRequest) -> Result<String> {
        let payment_method = self.get_payment_method(req.payment_method_id.clone()).await?;
        let provider = self.get_provider(payment_method.provider_id).await?;
        let mut conn = match self.pool.get() {
            Ok(c)=>c,
            Err(_)=>return Err(anyhow!("unable_to_get_conn"))
        };

        match provider.provider_type {
            PaymentProviderType::MobileMoney => {



                let resp = self.pretium.process(PretiumProcessRequest::OnRampMobile(OnRampRequestMobileReq {
                    phone: payment_method.identity,
                    network: provider.name,
                    amount: req.amount.to_string(),
                    currency_id: provider.supported_currency.symbol
                })).await?;



                match resp {
                    PretiumProcessResponse::OnRampMobile(d)=>{

                        diesel::insert_into(OnRampRequestsTable::table).values(&CreateOnRampRequest {
                            amount: BigDecimal::from(req.amount),
                            data: None,
                            requester: payment_method.owner,
                            transaction_ref: Some(d.transaction_code.clone()),
                            payment_method_id: payment_method.id,
                            target_token: req.target_token
                        }).returning(OnRampRequestsTable::dsl::id).get_result(&mut conn)?;

                        Ok(d.transaction_code.clone())
                    },
                    _=> Err(anyhow!("unsupported pretium response format"))
                }
            },
            _=>{
                 Err(anyhow!("payment_method_not_yet_supported"))
            }
        }
    }


    pub async fn handle_callback(&mut self, callback: TransactionCallbackData)->Result<()> {
        use crate::schema::on_ramp_requests::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(c)=>c,
            Err(e)=>{
                println!("Unable to resolve connection {e}");
                // silent fail as the caller is external
                return Ok(())
            }
        };

        let status_value = match callback.status {
            String::from("COMPLETE")=>OnRampRequestStatusEnum::Completed,
            String::from("FAILED")=>OnRampRequestStatusEnum::Failed,
            _=>OnRampRequestStatusEnum::Canceled
        };

        let data_json = match callback.receipt_number {
            Some(s)=>json!({
                "receipt": s
            }),
            None=>json!({})
        };

        let on_ramp_request = on_ramp_requests.filter(
            transaction_ref.eq(callback.transaction_code.clone())
        ).get_result::<GetOnRampRequest>(&mut conn)?;

        let method = self.get_payment_method(on_ramp_request.payment_method_id).await?;
        let provider = self.get_provider(method.provider_id).await?;


        let target_currency = match self.currencies.get_currency_by_id(on_ramp_request.target_token.clone()) {
            Some(c)=>c,
            None=>{
                println!("Target currency not found");
                // silent fail
                return Ok(())
            }
        };

        let token_b_amount = Currency::convert(&mut self.panora, &mut self.pretium, provider.supported_currency, target_currency.clone(), on_ramp_request.amount.unwrap().to_f64().unwrap()).await?;



        let hash =  self.req_handler.send(TumaRequest::Crypto(CryptoRequest {
            amount: token_b_amount,
            chain: TumaSupportedChains::APTOS(AptosWallet::new()?),
            token: target_currency,
            to: on_ramp_request.requester,
            on_ramp_request_id: on_ramp_request.id.to_string()
        })).await?;

        let _ = diesel::update(OnRampRequestsTable::table)
            .filter(
                transaction_ref.eq(callback.transaction_code).and(
                    status.eq(OnRampRequestStatusEnum::Pending)
                )
            )
            .set((
                status.eq(status_value),
                data.eq(data_json),
                final_token_quote.eq(BigDecimal::from(token_b_amount)),
                on_chain_transaction_hash.eq(hash)
                ))
            .execute(&mut conn)?;

        Ok(())
    }
}