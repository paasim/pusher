#!/usr/bin/env bash

set -eu

DB_NAME=$1
MIGRATIONS_DIR=migrations

_sqlite() {
  sqlite3 -bail -batch -init /dev/null -list -noheader "${DB_NAME}"
}

cat << SQL | _sqlite
CREATE TABLE IF NOT EXISTS __migrations(
  file_name TEXT PRIMARY KEY,
  succeeded INTEGER NOT NULL
) STRICT;
SQL

for fpath in "${MIGRATIONS_DIR}"/*.sql; do
  fname=$(basename "${fpath}")
  ex=$(echo "SELECT 1 FROM __migrations WHERE file_name='${fname}'" | _sqlite)

  if [ "1" != "${ex}" ]; then
    echo Running "${fname}"
    (echo "BEGIN;" && cat "${fpath}" && echo "; COMMIT;") | _sqlite \
      && echo "INSERT INTO __migrations VALUES('${fname}', $(date +%s))" | _sqlite
  fi
done
