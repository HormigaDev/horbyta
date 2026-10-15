#!/usr/bin/env python3
"""Ensaya backup/restauración solo en el PostgreSQL local del laboratorio."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import uuid

os.umask(0o077)
root = Path(__file__).resolve().parent.parent
state = json.loads((root / 'target/scope-tests/state.json').read_text())
schema = state['schema']
if not re.fullmatch(r'scope_test_[0-9a-f]{32}', schema):
    raise SystemExit('El archivo no identifica un esquema de pruebas válido.')
# Este script nunca consume DATABASE_URL ni el .env habitual.
base_url = 'postgresql://scope_test@127.0.0.1:55439/'
restore_db = 'scope_restore_' + uuid.uuid4().hex
output = root / 'target/scope-tests'
archive = output / 'backup.dump'
tables = ['horbyta_migrations', 'scope_projects', 'scope_revisions',
          'scope_change_revisions', 'scope_events', 'scope_effects',
          'scope_task_links', 'scope_notices', 'scope_delivery',
          'scope_preserved_attachments', 'change_requests', 'tasks',
          'milestones', 'invoices', 'payments', 'attachments',
          'scope_project_events', 'scope_project_event_attachments', 'scope_change_event_attachments', 'scope_task_requirements', 'project_messages', 'project_message_attachments', 'project_conversation_reads', 'conversation_change_sources']


def sql(database, statement, check=True):
    return subprocess.run(['psql', base_url + database, '-X', '-qAt',
                           '-v', 'ON_ERROR_STOP=1', '-c', statement],
                          check=check, capture_output=True, text=True)


def fingerprints(database):
    result = {}
    for table in tables:
        rows = json.loads(sql(database, f'SET search_path TO {schema}, public; '
                              f"SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]') FROM {table} t").stdout)
        canonical = json.dumps(rows, ensure_ascii=False, sort_keys=True).encode()
        result[table] = {'rows': len(rows), 'sha256': hashlib.sha256(canonical).hexdigest()}
    return result


before = fingerprints('postgres')
subprocess.run(['pg_dump', base_url + 'postgres', '--format=custom',
                '--schema=' + schema, '--no-owner', '--no-acl', '--file=' + str(archive)], check=True)
sql('postgres', f'CREATE DATABASE {restore_db}')
try:
    subprocess.run(['pg_restore', '--dbname=' + base_url + restore_db,
                    '--exit-on-error', '--no-owner', '--no-acl', str(archive)], check=True)
    after = fingerprints(restore_db)
    if before != after:
        raise SystemExit('La restauración no conserva los registros del laboratorio.')
    checksums = {}
    for version, scripts in [(1, [root / 'database/uuidv7.sql', root / 'database/schema.sql']), (2, [root / 'database/migrations/002_scope_control.sql']), (3, [root / 'database/migrations/003_scope_conversation.sql']), (4, [root / 'database/migrations/004_change_conversation.sql']), (5, [root / 'database/migrations/005_project_conversation.sql'])]:
        checksum = sql(restore_db, f'SET search_path TO {schema}, public; SELECT checksum FROM horbyta_migrations WHERE version={version}').stdout.strip()
        digest = hashlib.sha256()
        for script in scripts:
            data = script.read_bytes()
            digest.update(len(data).to_bytes(8, 'big'))
            digest.update(data)
        if checksum != digest.hexdigest():
            raise SystemExit(f'El checksum restaurado no coincide con la migración {version}.')
        checksums[version] = checksum
    guard = sql(restore_db, f"SET search_path TO {schema}, public; UPDATE scope_revisions SET document='{{}}'", check=False)
    if guard.returncode == 0 or 'scope_conflict:' not in guard.stderr:
        raise SystemExit('La restauración perdió la protección de las revisiones.')
    (output / 'backup-verification.json').write_text(json.dumps({
        'schema': schema, 'tables': after, 'migration_checksums': checksums,
        'immutable_revision_guard': True, 'restored_database_removed': True,
    }, indent=2))
    print(f'Restauración verificada: {len(tables)} tablas, migraciones e inmutabilidad conservadas.')
finally:
    sql('postgres', f'DROP DATABASE {restore_db}')
