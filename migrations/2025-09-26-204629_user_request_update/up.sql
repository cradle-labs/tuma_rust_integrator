-- Your SQL goes here
alter table user_request add column token text not null;
alter table user_request add column amount numeric;