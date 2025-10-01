-- Your SQL goes here
create table if not exists payment_sessions (
    id uuid primary key default uuid_generate_v4(),
    payment_provider_id text not null,
    payment_identity text not null,
    account_identity text,
    payer text not null,
    requested_at timestamp default now(),
    finalized_at timestamp,
    data jsonb,
    transaction_hash text,
    transferred_amount numeric not null default 0,
    transferred_token text,
    final_fiat_value numeric not null default 0,
    status offramp_request_status default 'pending',
    transaction_code text
);