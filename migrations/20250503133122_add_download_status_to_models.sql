-- Add migration script here
ALTER TABLE models
   ADD downloaded BOOLEAN DEFAULT FALSE;
