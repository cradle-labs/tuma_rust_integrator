use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use crate::schema::user_request as UserRequest;


#[derive(Deserialize, Serialize)]
pub struct CreateUserRequest {
    pub address: String,
    pub token: String,
    pub amount: BigDecimal
}
#[derive(Deserialize, Serialize, Insertable)]
#[diesel(table_name = UserRequest)]
pub struct CreateUser {
    pub address: String,
    pub token: String,
    pub amount: BigDecimal,
    pub transaction_code: String
}


#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PaymentResponseData {
    pub transaction_code: String,
    pub status: String,
    pub message: String
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PaymentResponse {
    pub code: u64,
    pub message: String,
    pub data: PaymentResponseData
}

#[derive(diesel_derive_enum::DbEnum, Debug, Serialize, Deserialize, Clone)]
#[ExistingTypePath = "crate::schema::sql_types::UserRequestStatus"]
#[serde(rename_all = "lowercase")]
pub enum UserRequestDataStatus {
    Pending,
    Completed,
    Failed
}


#[derive(Deserialize, Serialize, Clone, Debug, Queryable)]
pub struct UserRequestData {
    pub id: i32,
    pub address: String,
    pub requested_at: Option<NaiveDateTime>,
    pub status: Option<UserRequestDataStatus>,
    pub completed_at: Option<NaiveDateTime>,
    pub token: String,
    pub amount: Option<BigDecimal>,
    pub transaction_code: String
}