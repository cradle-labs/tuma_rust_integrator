use std::env;
use std::str::FromStr;
use diesel::{r2d2, ExpressionMethods, Insertable, PgConnection, QueryDsl, Queryable, RunQueryDsl};
use diesel::r2d2::ConnectionManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::{Result, anyhow};
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::NaiveDateTime;
use uuid::Uuid;
use crate::controller::aptos_panora_provider::AptosPanoraProvider;
use crate::controller::currency_controller::Currency;
use crate::payment_provider::provider::PaymentProviderType;
use crate::payment_provider::sender::FiatSender;
use crate::payment_provider::tuma_request_handler::{MobileFiatRequest, TumaRequest, TumaRequestHandler};
use crate::pretium::PretiumService;
use crate::r#static::currency::CurrencyStaticData;
use crate::r#static::providers::StaticProviderData;
use crate::schema::payment_sessions as PaymentsSessionTable;

/**
create table if not exists payment_sessions (
    id uuid primary key default uuid_generate_v4(),
    payment_provider_id text not null,
    payment_identity text not null,
    account_identity text,
    payer text not null,
    requested_at timestamp default now(),
    finalized_at timestamp,
    data jsonb,
    transaction_hash text,
    transferred_amount numeric not null default 0,
    transferred_token text,
    final_fiat_value numeric not null default 0
);
**/
#[derive(Serialize, Deserialize, Insertable)]
#[diesel(table_name=PaymentsSessionTable)]
pub struct CreatePaymentSession {
    pub payment_provider_id: String,
    pub payment_identity: String,
    pub account_identity: Option<String>,
    pub payer: String,
    pub data: Option<Value>,
    pub transferred_token: String,
}



#[derive(Serialize,Deserialize,Clone,Debug, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::OfframpRequestStatus"]
pub enum OffRampStatus {
    Pending,
    Completed,
    Failed
}


#[derive(Serialize, Deserialize, Queryable)]
#[diesel(table_name=PaymentsSessionTable)]
pub struct GetPaymentSession {
    pub id: Uuid,
    pub payment_provider_id: String,
    pub payment_identity: String,
    pub account_identity: Option<String>,
    pub payer: String,
    pub requested_at: Option<NaiveDateTime>,
    pub finalized_at: Option<NaiveDateTime>,
    pub data: Option<Value>,
    pub transaction_hash: Option<String>,
    pub transferred_amount: BigDecimal,
    pub transferred_token: Option<String>,
    pub final_fiat_value: BigDecimal,
    pub status: Option<OffRampStatus>,
    pub transaction_code: Option<String>
}

pub struct PaymentSessions {
    pub pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    pub pretium_service: PretiumService,
    pub panora: AptosPanoraProvider,
    pub handler: TumaRequestHandler,
    pub currencies: CurrencyStaticData,
    pub providers: StaticProviderData
}


impl PaymentSessions {
    pub fn new(pool: r2d2::Pool<ConnectionManager<PgConnection>>) -> Result<Self> {
        let api_key = env::var("PRETIUM_API_KEY")?;
        let pretium_service = PretiumService::new(api_key)?;
        let fiat_sender = FiatSender::new(pretium_service.clone());
        let handler = TumaRequestHandler::new(pool.clone(), fiat_sender);
        let panora = AptosPanoraProvider::new();
        Ok(Self {
            pool,
            pretium_service,
            panora,
            handler,
            currencies: CurrencyStaticData::new(),
            providers: StaticProviderData::new()
        })
    }


    pub async fn create_payment_session(&mut self, payer: String, provider: String, receiver_id: String, token: String, account_identity: Option<String>) -> Result<Uuid> {
        let mut conn = self.pool.get()?;

        let id = diesel::insert_into(PaymentsSessionTable::table).values(& CreatePaymentSession {
            payment_identity: receiver_id,
            payment_provider_id: provider,
            data: None,
            payer,
            account_identity,
            transferred_token: token,
        }).returning(PaymentsSessionTable::id).get_result::<(Uuid)>(&mut conn)?;

       Ok(id)

    }


    pub async fn off_ramp_payment_session(&mut self, session_id: String, token_amount: f64, token_address: String, transaction_hash_value: String)-> Result<Uuid> {
        let session_id_as_uuid = Uuid::from_str(session_id.as_str())?;
        let mut conn = self.pool.get()?;

        use crate::schema::payment_sessions::dsl::*;


        let session = payment_sessions.find(session_id_as_uuid).first::<GetPaymentSession> (&mut conn)?;
        let provider = match self.providers.get_id(session.payment_provider_id.as_str()) {
            Some(v)=>v,
            None=>return Err(anyhow!("Unable to obtain provider"))
        };

        let token_a_currency = match self.currencies.get_currency_by_token(token_address.to_string()) {
            Some(v)=>v,
            None=>return Err(anyhow!("Currency for provided token address not yet supported"))
        };

        let token_b_currency = provider.supported_currency.clone();

        let token_a_amount = token_amount.clone();

        let token_b_amount = Currency::convert(&mut self.panora.clone(), &mut self.pretium_service, token_a_currency.clone(), token_b_currency.clone(), token_a_amount).await?;


        let req = match provider.provider_type {
            PaymentProviderType::MobileMoney => {
                match session.account_identity {
                    Some(user_account)=>{
                        todo!("Add support for paybill transactions")
                    },
                    None=>{
                        TumaRequest::MobileFiat(MobileFiatRequest {
                            currency: token_b_currency,
                            amount: token_b_amount,
                            number: session.payment_identity,
                            network_id: provider.id
                        })
                    }
                }
            },
            PaymentProviderType::Bank=>{
                todo!("bank payments are pending")
            }
        };


        let transaction_code_value = self.handler.send(req).await?;

        let token_a_big = BigDecimal::from_f64(token_a_amount);

        let  res = diesel::update(PaymentsSessionTable::table).filter(
            id.eq(session_id_as_uuid)
        ).set(
            (
                transaction_hash.eq(transaction_hash_value),
                transaction_code.eq(transaction_code_value),
                transferred_amount.eq(BigDecimal::from_f64(token_a_amount).unwrap()),
                final_fiat_value.eq(BigDecimal::from_f64(token_b_amount).unwrap())
            )
        ).execute(&mut conn)?;

        Ok(session_id_as_uuid)
    }
}