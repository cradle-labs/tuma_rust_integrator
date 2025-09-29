-- Your SQL goes here
alter table on_ramp_requests add column if not exists final_token_quote numeric default 0;