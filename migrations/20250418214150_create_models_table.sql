-- Add migration script here
CREATE TYPE ModelType as ENUM('image', 'text');

CREATE TABLE IF NOT EXISTS models (
  model_id uuid, 
  model_type ModelType
);
