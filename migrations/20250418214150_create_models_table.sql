-- Add migration script here
CREATE TYPE modelspec as (model_id TEXT, model_type integer);

CREATE TABLE IF NOT EXISTS models (
  spec modelspec,
  UNIQUE(spec)
);
