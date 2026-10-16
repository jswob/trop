#!/bin/sh
set -e

# Migrate before starting, unless the host has done it already: Fly migrates in its
# release_command and sets MIGRATE_ON_START=false, so a machine waking from zero starts at once.
if [ "${MIGRATE_ON_START:-true}" != "false" ]; then
	echo "Running database migrations..."
	bun src/db/migrate.ts
fi

echo "Starting API server..."
exec bun src/index.ts
