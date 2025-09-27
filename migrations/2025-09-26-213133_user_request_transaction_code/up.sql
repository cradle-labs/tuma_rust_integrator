-- Your SQL goes here
alter table user_request add column transaction_code text not null unique;