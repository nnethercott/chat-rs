-- Add migration script here
BEGIN; 
  DROP TABLE models;
  DROP TYPE modelspec;

  CREATE TABLE IF NOT EXISTS models (
    model_id TEXT PRIMARY KEY,
    model_type integer
  );

COMMIT;
