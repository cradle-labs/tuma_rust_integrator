-- Your SQL goes here

create type user_request_status as enum (
    'pending',
    'completed',
    'failed'
);

alter table user_request  add column status user_request_status default 'pending';
alter table user_request add column completed_at timestamp;