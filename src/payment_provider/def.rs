use std::collections::HashMap;
use diesel::{r2d2, PgConnection, RunQueryDsl};
use diesel::r2d2::ConnectionManager;
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value::String;
use crate::payment_provider::data::{CreateUser, CreateUserRequest, PaymentResponse, UserRequestData, UserRequestDataStatus};
use crate::schema::user_request as UserRequest;
use diesel::prelude::*;
use crate::schema::user_request::transaction_code;

#[derive(Clone)]
pub struct PaymentProvider {
    pub pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    pub api_key: String
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum PStatus {
    COMPLETE,
    FAILED
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PaymentStatusCallback {
    pub status: PStatus,
    pub transaction_code: String,
    pub message: String,
    pub public_name: Option<String>,
    pub receipt_number: Option<String>
}

impl PaymentProvider {
    pub fn new(pool: r2d2::Pool<ConnectionManager<PgConnection>>, api_key: String)-> Self {
        Self {
            pool,
            api_key
        }
    }

    pub async fn send_payment_request(&mut self, req: &CreateUserRequest) -> Result<PaymentResponse> {
        let client = Client::new();

        let mut payload = HashMap::new();
        payload.insert("shortcode", ""); // TODO: add payments methods table with these details
        payload.insert("amount", &req.amount.to_string());
        payload.insert("mobile_network", "Safaricom");
        payload.insert("callback_url", ""); // TODO: set callback url


        let resp = client.post("")
            .header("x-api-key", self.api_key.clone())
            .json(&payload)
            .send()
            .await?
            .json::<PaymentResponse>()
            .await?;


        Ok(resp)
    }

    pub async fn create_request(&mut self, req: CreateUserRequest) ->Result<()> {
        let mut conn = match self.pool.get() {
            Ok(c)=>c,
            Err(e)=>{
                println!("Something went wrong :: {}",e);
                return Err(anyhow!("Unable to create connection"))
            }
        };

        let tx_request = self.send_payment_request(&req).await?;



        diesel::insert_into(UserRequest::table).values( CreateUser {
            transaction_code: tx_request.data.transaction_code,
            address: req.address,
            amount: req.amount,
            token: req.token
        }).execute(&mut conn)?;

        Ok(())
    }


    pub async fn handle_callback(&mut self, callback_data: PaymentStatusCallback) -> Result<()> {
        let mut conn = match self.pool.get() {
            Ok(c)=>c,
            Err(e)=>{
                println!("Something went wrong :: {}",e);
                return Err(anyhow!("Unable to create connection"))
            }
        };

        use crate::schema::user_request::dsl::*;



        let existing = match user_request.filter(
            transaction_code.eq(callback_data.transaction_code)
        ).get_result::<UserRequestData>(&mut conn) {
            Ok(res)=>res,
            Err(e)=>{
                println!("Something went wrong {:?}",e);
                return Err(anyhow!("Double submission attempted");
            }
        };

        match existing.status {
            Some(t)=>{
                match t {
                    UserRequestDataStatus::Pending=>{
                        // do nothing, means we can proceed
                    },
                    _=>{
                        return Err(anyhow!("transaction already confirmed"))
                    }
                }
            },
            None=>{
                println!("No existing data");
                return Err(anyhow!("Status not provided"))
            }
        }


        // issue the funds then send out the crypto the payment
        // TODO: send out the crypto




        let result = diesel::update(user_request).filter(transaction_code.eq(callback_data.transaction_code)).set(
            status.eq(UserRequestDataStatus::Completed))
            .execute(&mut conn)?;


        Ok(())
    }


    pub async fn disburse_fiat(&mut self) -> Result<()> {
        Ok(())
    }





}
