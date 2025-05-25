#!/bin/bash

set -x 
set -eo pipefail

if [[ ! -x $(command -v redis-cli) ]]; then
  echo "make sure redis-cli is installed!" 
  exit 1
fi

REDISCLI_AUTH="${REDISCLI_AUTH:=password}"

docker run \
  --restart always \
  -d \
  -p 6379:6379 \
  redis:latest \
  redis-server --requirepass "${REDISCLI_AUTH}"

# sleep until ready
until redis-cli -a ${REDISCLI_AUTH} PING; do
  echo "connecting to redis ..." 
  sleep 1
done
