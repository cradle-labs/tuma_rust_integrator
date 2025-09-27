-- Your SQL goes here

create type ledger_entry_type as enum (
    'on_chain',
    'off_chain'
);

create table if not exists ledger (
    id serial primary key,
    from_account text not null ,
    to_account text not null ,
    token text not null,
    amount numeric not null,
    created_at timestamp default now(),
    entry_type ledger_entry_type not null,
    data jsonb,
    transaction_hash text
);

