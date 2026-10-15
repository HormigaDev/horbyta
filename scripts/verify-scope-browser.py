#!/usr/bin/env python3
"""Servidor y navegador de laboratorio dentro de una única cuota de recursos."""
import os
from pathlib import Path
import subprocess
import time
import urllib.request
import urllib.error

root = Path(__file__).resolve().parent.parent
os.chdir(root)
env = os.environ.copy()
# No imprimir secretos; conexión usada únicamente con search_path del laboratorio.
for line in (root / '.env').read_text().splitlines():
    if line.startswith('DATABASE_URL='):
        env['SCOPE_TEST_DATABASE_URL'] = line.split('=', 1)[1].strip().strip('"').strip("'")
env.update(CHROME_LOW_THREAD='1', UI_API_PORT='8897', UI_VERIFY_SCOPE_REALTIME='0' if env.get('UI_VERIFY_DOCUMENT_PREVIEW') == '1' or env.get('UI_VERIFY_PROJECT_LAYOUT') == '1' or env.get('UI_VERIFY_PROJECT_CHAT') == '1' or env.get('UI_VERIFY_SCOPE_NOTIFICATION') == '1' or env.get('UI_VERIFY_CHANGE_UX') == '1' else '1',
           UI_SCREENSHOT_DIR='target/scope-tests/screenshots')
if not (root / "frontend/dist/pwa/index.html").is_file():
    raise SystemExit("Falta el build del frontend; ejecutar build antes del navegador.")
processes = []
logs = []
try:
    for label, args in [('backend', ['python3','scripts/start-scope-test-server.py']),
                        ('frontend', ['node','scripts/serve-ui-build.mjs','frontend/dist/pwa','9107'])]:
        path = root / 'target/scope-tests' / f'{label}.log'
        fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
        log = os.fdopen(fd, 'w')
        logs.append(log)
        processes.append(subprocess.Popen(args, env=env, stdout=log, stderr=log))
    for port, route in [(8897, '/api/v1/health'), (9107, '/')]:
        deadline = time.monotonic() + 30
        while True:
            if any(p.poll() is not None for p in processes):
                raise SystemExit('El servidor aislado falló; consultar logs privados del laboratorio.')
            try:
                urllib.request.urlopen(f'http://127.0.0.1:{port}{route}', timeout=1)
                break
            except urllib.error.HTTPError:
                break
            except Exception:
                if time.monotonic() >= deadline:
                    raise SystemExit('Tiempo de espera al arrancar laboratorio.')
                time.sleep(.2)
    result = subprocess.run(['node','scripts/capture-ui.mjs','http://127.0.0.1:9107','/projects'], env=env)
    raise SystemExit(result.returncode)
finally:
    for p in processes:
        p.terminate()
    for p in processes:
        try:
            p.wait(timeout=5)
        except subprocess.TimeoutExpired:
            p.kill(); p.wait()
    for log in logs:
        log.close()
