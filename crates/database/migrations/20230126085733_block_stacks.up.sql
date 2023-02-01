-- Add up migration script here

CREATE TABLE IF NOT EXISTS block_stack
(
    id                 SERIAL PRIMARY KEY,
    address            VARCHAR(66) NOT NULL,
    name               VARCHAR(32) NOT NULL,
    stack              jsonb       NOT NULL,
    last_edit_datetime TIMESTAMP   NOT NULL DEFAULT current_timestamp,
    bytecode           bytea,

    UNIQUE (address, name)
);

CREATE INDEX idx_block_stack_address ON block_stack (address);
