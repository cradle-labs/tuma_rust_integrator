-- Your SQL goes here
create table if not exists user_accounts (
    address text primary key,
    created_at timestamp default now(),
    phone_number text unique not null,
    country text not null
)