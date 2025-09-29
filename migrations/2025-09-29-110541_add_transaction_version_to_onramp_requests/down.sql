-- This file should undo anything in `up.sql`
alter table on_ramp_requests drop column if exists on_chain_transaction_hash;