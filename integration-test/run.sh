#!/bin/bash

if [ -z "${PIPENV_ACTIVE}" ]; then
  echo "Installing integration test dependencies"
  pipenv sync
  source "$(pipenv --venv)/bin/activate"
fi

echo "Bringing docker compose up"
docker compose --project-name integration-test --file ../docker-compose.yml up -d

echo "Running Behave"
behave "$@" --no-skipped --format pretty

echo "Bringing docker compose down"
docker compose down
