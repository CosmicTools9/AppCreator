#!/bin/bash
# ensure-schema.sh — Initialize the PostgreSQL schema required by AppCreator's AppAgent.
#
# AppCreator is an independently open-sourced product that reuses AliothStudio's
# AppAgent engine. AppAgent reads from `isahl_meta` and `isahl` schemas, so the
# target database must contain the same schema that AliothStudio Meta uses.
#
# This script delegates schema initialization to AliothStudio's canonical
# `scripts/db/reset-db.sh`, which applies the latest backup schema
# (`Backup/latest/schema.sql`). Per ENVIRONMENT_SPEC.md, DB is the schema truth
# source; we do not maintain a forked DDL copy in AppCreator.
#
# Usage:
#   ALIOTH_STUDIO_ROOT=/path/to/AliothStudio DATABASE_URL=postgres://... \
#     bash scripts/db/ensure-schema.sh
#
# Optional:
#   RESET=true bash scripts/db/ensure-schema.sh   # drop and recreate the DB first

set -euo pipefail

ALIOTH_STUDIO_ROOT="${ALIOTH_STUDIO_ROOT:-}"
DATABASE_URL="${DATABASE_URL:-}"
RESET="${RESET:-false}"

if [[ -z "$ALIOTH_STUDIO_ROOT" ]]; then
    echo "ERROR: ALIOTH_STUDIO_ROOT must be set to the AliothStudio checkout path."
    echo "Example: ALIOTH_STUDIO_ROOT=/path/to/AliothStudio bash scripts/db/ensure-schema.sh"
    exit 1
fi

if [[ -z "$DATABASE_URL" ]]; then
    echo "ERROR: DATABASE_URL must be set."
    exit 1
fi

# Validate tier: refuse production / pre-release.
DB_NAME=$(echo "$DATABASE_URL" | sed -E 's|.*/([^/]+)$|\1|')
if [[ "$DB_NAME" == "aliothstudio" || "$DB_NAME" == "aliothstudio_pre" ]]; then
    echo "ERROR: DATABASE_URL targets '$DB_NAME'."
    echo "AppCreator MUST only use aliothstudio_dev, aliothstudio_test, or a dedicated database."
    exit 1
fi

RESET_DB_SCRIPT="$ALIOTH_STUDIO_ROOT/scripts/db/reset-db.sh"
if [[ ! -f "$RESET_DB_SCRIPT" ]]; then
    echo "ERROR: reset-db.sh not found at $RESET_DB_SCRIPT"
    echo "Please ensure ALIOTH_STUDIO_ROOT points to a valid AliothStudio checkout."
    exit 1
fi

echo "=== AppCreator schema initialization ==="
echo "AliothStudio root: $ALIOTH_STUDIO_ROOT"
echo "Target database:   $DB_NAME"

declare -a ARGS
if [[ "$RESET" == "true" ]]; then
    ARGS=("--reset")
fi

# reset-db.sh reads DATABASE_URL and applies Backup/latest/schema.sql (INIT mode).
DATABASE_URL="$DATABASE_URL" bash "$RESET_DB_SCRIPT" ${ARGS[@]+"${ARGS[@]}"}

echo "=== Schema initialization complete ==="
