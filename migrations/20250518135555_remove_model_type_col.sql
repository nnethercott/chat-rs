BEGIN;
  DROP TABLE IF EXISTS models;
  DROP TYPE IF EXISTS modelspec;

  CREATE TYPE modelspec AS (
    model_id TEXT
  );

  -- Recreate the table
  CREATE TABLE IF NOT EXISTS models (
    spec modelspec,
    UNIQUE(spec)
  );
COMMIT;
