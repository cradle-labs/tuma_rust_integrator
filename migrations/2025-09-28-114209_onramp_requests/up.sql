-- Your SQL goes here

create type onramp_request_status as enum (
  'pending',
    'completed',
    'failed',
    'canceled'
);

create table if not exists on_ramp_requests (
    id uuid primary key default uuid_generate_v4(),
    requester text not null references account(address),
    payment_method_id uuid not null references payment_method(id),
    status onramp_request_status not null default 'pending',
    transaction_ref text,
    data jsonb,
    amount numeric,
    requested_at timestamp not null  default now(),
    finalized_at timestamp
);