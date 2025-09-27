-- Your SQL goes here
create table if not exists user_request (
    id serial primary key,
    address text not null,
    requested_at timestamp default now()
)