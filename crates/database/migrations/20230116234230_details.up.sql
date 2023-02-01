-- Add up migration script here

CREATE TABLE IF NOT EXISTS account_detail
(
    id      SERIAL PRIMARY KEY,
    address VARCHAR(66) NOT NULL,
    alias   VARCHAR(256)
);

CREATE INDEX idx_account_detail_address on account_detail (address);

CREATE TABLE IF NOT EXISTS module_detail
(
    id          SERIAL PRIMARY KEY,
    address     VARCHAR(66)  NOT NULL,
    module_name VARCHAR(128) NOT NULL,
    description TEXT,
    github_url  TEXT,
    rev         TEXT default 'main',
    subdir      TEXT default null
);

CREATE TABLE IF NOT EXISTS module_function_detail
(
    id                  SERIAL PRIMARY KEY,
    address             VARCHAR(66)  NOT NULL,
    module_name         VARCHAR(128) NOT NULL,
    function_name       TEXT         NOT NULL,
    description         TEXT,
    param_names         jsonb,
    generic_type_params jsonb
);