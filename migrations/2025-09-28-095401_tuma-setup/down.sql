-- This file should undo anything in `up.sql`
-- More explicit version with CASCADE to handle dependencies
drop table if exists ledger cascade;
drop table if exists payment_method cascade;
drop table if exists account cascade;

drop type if exists transaction_type;
drop type if exists ledger_entry_type;
drop type if exists payment_method_type;