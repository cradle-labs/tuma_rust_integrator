// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ledger_entry_type"))]
    pub struct LedgerEntryType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_request_status"))]
    pub struct UserRequestStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LedgerEntryType;

    ledger (id) {
        id -> Int4,
        from_account -> Text,
        to_account -> Text,
        token -> Text,
        amount -> Numeric,
        created_at -> Nullable<Timestamp>,
        entry_type -> LedgerEntryType,
        data -> Nullable<Jsonb>,
        transaction_hash -> Nullable<Text>,
    }
}

diesel::table! {
    user_accounts (address) {
        address -> Text,
        created_at -> Nullable<Timestamp>,
        phone_number -> Text,
        country -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserRequestStatus;

    user_request (id) {
        id -> Int4,
        address -> Text,
        requested_at -> Nullable<Timestamp>,
        status -> Nullable<UserRequestStatus>,
        completed_at -> Nullable<Timestamp>,
        token -> Text,
        amount -> Nullable<Numeric>,
        transaction_code -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    ledger,
    user_accounts,
    user_request,
);
