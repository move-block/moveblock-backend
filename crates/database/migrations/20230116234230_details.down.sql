-- Add down migration script here

DROP INDEX IF EXISTS idx_account_detail_address;

DROP TABLE IF EXISTS account_detail;
DROP TABLE IF EXISTS module_detail;
DROP TABLE IF EXISTS module_function_detail;