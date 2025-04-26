#!/bin/bash
set -x 
set -eo pipefail

if [[ ! -x $(command -v sqlx) ]]; then
  echo "make sure sqlx is installed!" 
  exit 1
fi

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=models}"

docker run \
  -d \
  -e POSTGRES_USER=${DB_USER} \
  -e POSTGRES_PASSWORD=${DB_PASSWORD} \
  -e POSTGRES_DB=${DB_NAME} \
  -p 5432:5432 \
  postgres:latest

# sleep until ready
export PGPASSWORD=${POSTGRES_PASSWORD}
until psql -h 0.0.0.0 -p 5432 -U ${DB_USER} -d ${DB_NAME} -c '\q'; do
  echo "database not yet ready! sleeping ..." 
  sleep 1
done

export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@localhost:5432/${DB_NAME}"
sqlx migrate run
