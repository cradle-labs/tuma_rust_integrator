-- This file should undo anything in `up.sql`
drop table if exists on_ramp_requests cascade;
drop type if exists onramp_request_status;