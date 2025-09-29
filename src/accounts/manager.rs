use diesel::{r2d2, Insertable, PgConnection};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use serde::{Deserialize, Serialize};
use crate::schema::account as AccountTable;
use crate::schema::payment_method as PaymentMethodTable;
use diesel::prelude::*;
use anyhow::{Result, anyhow};
use uuid::Uuid;

pub struct AccountManager {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>
}


#[derive(Deserialize,Serialize, Insertable)]
#[diesel(table_name = AccountTable)]
pub struct CreateAccountReq {
    pub address: String
}

#[derive(Deserialize, Serialize, diesel_derive_enum::DbEnum, Debug)]
#[ExistingTypePath = "crate::schema::sql_types::PaymentMethodType"]
pub enum PaymentMethodType {
    Bank,
    #[serde(rename = "mobile-money")]
    MobileMoney
}

#[derive(Deserialize, Serialize, Insertable)]
#[diesel(table_name = PaymentMethodTable)]
pub struct CreatePaymentMethod {
    pub owner: String,
    pub payment_method_type: PaymentMethodType,
    pub identity: String,
    pub provider_id: String
}

impl AccountManager {
    pub fn new(pool: r2d2::Pool<ConnectionManager<PgConnection>>)-> Self {
        Self {
            pool
        }
    }

    pub async fn create(&mut self, req: CreateAccountReq)-> Result<String>{
        use crate::schema::account::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) =>conn,
            Err(e)=>{
                println!("Unable to get connection {}",e);
                return Err(anyhow!("unable to create a connection"))
            }
        };

        let inserted_address = diesel::insert_into(AccountTable::table).values(&req).returning(address).get_result::<(String)>(&mut conn)?;

        Ok(inserted_address)
    }

    pub async fn add_payment_method(&mut self, req: CreatePaymentMethod)->Result<String> {
        use crate::schema::payment_method::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn)=> conn,
            Err(e)=>{
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };


        let inserted_id = diesel::insert_into(PaymentMethodTable::table).values(&req).returning(id).get_result::<(Uuid)>(&mut conn)?;
        let cloned = inserted_id.clone();
        Ok(cloned.to_string())
    }

}