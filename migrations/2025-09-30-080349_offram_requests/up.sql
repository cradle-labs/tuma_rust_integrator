-- Your SQL goes here

create type offramp_request_status as enum (
    'pending',
    'completed',
    'failed'
    );

create table if not exists off_ramp_requests (
    id uuid primary key default uuid_generate_v4(),
    requester text not null references account(address),
    from_token text not null,
    from_token_amount numeric not null,
    transaction_version text not null,
    transaction_hash text not null,
    transaction_code text,
    data jsonb,
    requested_at timestamp not null default now(),
    finalized_at timestamp,
    status offramp_request_status not null default 'pending'
)