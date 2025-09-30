-- This file should undo anything in `up.sql`
drop table if exists off_ramp_requests cascade;
drop type if exists offramp_request_status;