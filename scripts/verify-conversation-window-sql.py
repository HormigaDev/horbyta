#!/usr/bin/env python3
"""Valida el SQL real de ventanas en el laboratorio sin enlazar binarios Rust."""
import json
import os
from pathlib import Path
import re
import subprocess
import urllib.parse

root = Path(__file__).resolve().parent.parent
state = json.loads((root/'target/scope-tests/state.json').read_text())
assert re.fullmatch(r'scope_test_[0-9a-f]{32}',state['schema'])
url = os.environ.get('SCOPE_TEST_DATABASE_URL')
if not url:
    for line in (root/'.env').read_text().splitlines():
        if line.startswith('DATABASE_URL='):
            url = line.split('=',1)[1].strip().strip('"').strip("'")
p = urllib.parse.urlsplit(url)
env = os.environ.copy()
env.update(PGHOST=p.hostname,PGPORT=str(p.port or 5432),PGDATABASE=p.path.lstrip('/'),
           PGUSER=urllib.parse.unquote(p.username),PGPASSWORD=urllib.parse.unquote(p.password or ''))
for key,value in urllib.parse.parse_qsl(p.query):
    if key=='sslmode': env['PGSSLMODE']=value
source = (root/'src/services/scope_conversation.rs').read_text()
segment = re.search(r'let segment = if around.is_some\(\) \{\s*"(.*?)"\.to_owned\(\)',source,re.S).group(1)
query = re.search(r'r#"(WITH segment.*?)"#',source,re.S).group(1).replace('{segment}',segment)
owner = state['owner']; project = state['conversation_project']
sql = f'''BEGIN;
SET LOCAL search_path TO {state['schema']},public;
SELECT id AS anchor FROM scope_project_events WHERE project_id='{project}' AND revision=1 ORDER BY recorded_at,id OFFSET 99 LIMIT 1 \\gset
PREPARE window_query(uuid,integer,uuid,uuid,uuid,uuid) AS {query};
EXECUTE window_query('{project}',1,NULL,NULL,'{owner}',:'anchor');
ROLLBACK;
'''
def run(sql):
    r = subprocess.run(['psql','-X','-qAt','-v','ON_ERROR_STOP=1'],input=sql,text=True,env=env,capture_output=True)
    if r.returncode:
        # SQL y errores no contienen credenciales; limitar salida.
        raise SystemExit(r.stderr[-1500:])
    return [json.loads(line) for line in r.stdout.splitlines() if line.startswith('{')]
result = run(sql)[0]
assert len(result['items']) == 100
assert any(m['id']==result['focus_id'] for m in result['items'])
assert result['has_older'] and result['has_newer']
legacy_query = re.search(r'"(SELECT CASE WHEN count\(\*\)=1.*?)",',source,re.S).group(1)
sql=f'''BEGIN;
SET LOCAL search_path TO {state['schema']},public;
INSERT INTO notifications(user_id,preference_key,event_type,title,body,payload)
VALUES('{owner}','client_portal.change_request_updated','client_portal.change_request_updated','Aviso de laboratorio','nota.md',jsonb_build_object('project_id','{project}','revision',1)) RETURNING id AS notice \\gset
PREPARE legacy_query(uuid,uuid,integer,uuid) AS SELECT jsonb_build_object('focus',r.id) FROM ({legacy_query}) r;
EXECUTE legacy_query(:'notice','{project}',1,'{owner}');
EXECUTE legacy_query(:'notice','{project}',1,'00000000-0000-4000-8000-000000000001');
ROLLBACK;
'''
legacy,forbidden=run(sql)
assert legacy['focus'] is not None
assert forbidden['focus'] is None
print('SQL real: ventana anclada de 100, mensaje incluido, flags y resolución de aviso antiguo con aislamiento de propietario: OK')
