-- Your SQL goes here
alter table on_ramp_requests add column if not exists target_token text not null;