-- Add down migration script here

DROP INDEX IF EXISTS idx_block_stack_address;
DROP TABLE IF EXISTS block_stack;