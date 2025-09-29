-- Your SQL goes here
create extension if not exists "uuid-ossp";

create table if not exists account (
    address text not null unique primary key,
    created_at timestamp not null default now()
);

create type payment_method_type as enum (
    'bank',
    'mobile-money'
);

create table if not exists payment_method
(
    id                  uuid primary key    default uuid_generate_v4(),
    owner               text not null references account(address),
    payment_method_type payment_method_type default 'mobile-money',
    identity            text not null,
    provider_id         text not null,
    created_at           timestamp not null default now()
);


create type ledger_entry_type as enum (
    'on-chain',
    'off-chain'
);

create type transaction_type as enum (
    'deposit',
    'withdrawal'
);


create table if not exists ledger
(
    id uuid primary key default uuid_generate_v4(),
    address text not null references account(address),
    entry_type ledger_entry_type default 'on-chain',
    on_chain_transaction_version numeric,
    off_chain_transaction_hash text,
    transaction_type transaction_type default 'deposit',
    payment_method_id uuid references payment_method(id),
    timestamp timestamp default now()
)

