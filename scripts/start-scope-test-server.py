#!/usr/bin/env python3
"""Inicia el backend únicamente contra el esquema desechable de las pruebas."""
import json
import os
from pathlib import Path
import re
import secrets
import sys
import urllib.parse

root = Path(__file__).resolve().parent.parent
state = json.loads((root / 'target/scope-tests/state.json').read_text())
schema = state['schema']
if not re.fullmatch(r'scope_test_[0-9a-f]{32}', schema):
    sys.exit('El archivo no identifica un esquema de pruebas válido.')
base_url = os.environ.get('SCOPE_TEST_DATABASE_URL', 'postgresql://scope_test@127.0.0.1:55439/postgres')
parsed = urllib.parse.urlsplit(base_url)
query = dict(urllib.parse.parse_qsl(parsed.query))
query['options'] = f'-csearch_path={schema},public'
os.environ['DATABASE_URL'] = urllib.parse.urlunsplit(parsed._replace(query=urllib.parse.urlencode(query)))
os.environ['JWT_SECRET'] = secrets.token_hex(32)
os.environ['JWT_SECRET_CLIENT'] = secrets.token_hex(32)
os.environ['APP_ENV'] = 'development'
os.environ['APP_URI'] = '127.0.0.1:8897'
os.environ['FRONTEND_ORIGIN'] = 'http://127.0.0.1:9107'
os.environ['TIME_TO_EXP_SESSION'] = '2h'
os.environ['ALLOW_REGISTRATION'] = 'false'
os.environ['TOKIO_WORKER_THREADS'] = '1'
for key in list(os.environ):
    if key.startswith('S3_'):
        os.environ.pop(key)
runtime = Path('/tmp') / f'horbyta-runtime-{schema}'
runtime.mkdir(exist_ok=True)
os.environ['UPLOAD_DIR'] = str(runtime / 'uploads')
# La aplicación no encuentra el .env habitual al iniciar aquí.
os.chdir(runtime)
os.execv(root / 'target/debug/horbyta', ['horbyta'])
