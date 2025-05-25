BEGIN;
  DROP TABLE IF EXISTS models;
  DROP TYPE IF EXISTS modelspec;

  -- Recreate the table
  CREATE TABLE IF NOT EXISTS models (
    model_id text,
    UNIQUE(model_id)
  );
COMMIT;
