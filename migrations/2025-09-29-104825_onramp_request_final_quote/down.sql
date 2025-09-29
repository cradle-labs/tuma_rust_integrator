-- This file should undo anything in `up.sql`
alter table on_ramp_requests drop column if exists final_token_quote;