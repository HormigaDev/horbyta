import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';

// Opt-in integration checks against the dedicated local UI test account.
export async function verify({ call, evaluate, navigate, waitFor, output }) {
  const checks = [];
  const shot = async file => {
    const { data } = await call('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
    await writeFile(join(output, file), Buffer.from(data, 'base64'));
  };
  const check = async (label, expression) => {
    if (!await evaluate(expression)) throw new Error(`UI verification failed: ${label}`);
    checks.push(label);
    console.log(`Verified: ${label}`);
  };
  if (!await evaluate("fetch('/api/v1/users/me').then(r=>r.json()).then(u=>u.email === 'ui-tests@horbyta.test')")) throw new Error('Verification requires the dedicated ui-tests@horbyta.test account.');
  const original = await evaluate("fetch('/api/v1/users/me').then(r=>r.json()).then(u=>u.preferences)");
  let movedTaskId;
  try {
    await navigate('/settings');
    await check('Appearance selector is rendered', "document.querySelectorAll('.theme-choice').length === 3");
    await evaluate("document.querySelector('[data-preview-id=forest]').click()");
    await waitFor("document.documentElement.dataset.themeId === 'forest'", 'appearance preview');
    await check('Preview leaves server unchanged', `fetch('/api/v1/users/me').then(r=>r.json()).then(u=>(u.preferences.appearance?.theme_id ?? 'orbit') === ${JSON.stringify(original.appearance?.theme_id ?? 'orbit')})`);
    await evaluate("Array.from(document.querySelectorAll('.q-card__actions button')).find(b=>b.textContent.includes('Cancelar') || b.textContent.includes('Cancel') || b.textContent.includes('Cancelar')).click()");
    await waitFor(`document.documentElement.dataset.themeId === ${JSON.stringify(original.appearance?.theme_id ?? 'orbit')}`, 'cancel preview');
    checks.push('Cancel restores confirmed appearance');
    await evaluate("document.querySelector('[data-preview-id=linen]').click()");
    await evaluate(`(() => {
      const originalFetch = window.fetch;
      window.__restoreFetch = () => { window.fetch = originalFetch; };
      window.fetch = (input, init) => String(input).endsWith('/users/me/preferences') && init?.method === 'PATCH'
        ? Promise.resolve(new Response(JSON.stringify({ error: 'test failure', type: 'internal' }), { status: 500, headers: { 'Content-Type': 'application/json' } }))
        : originalFetch(input, init);
      Array.from(document.querySelectorAll('.q-card__actions button')).find(b=>b.textContent.includes('Guardar preferencias')).click();
    })()`);
    await waitFor("!!document.querySelector('.q-notification')", 'failed save notification');
    await check('Failed appearance save keeps unsaved preview', "document.documentElement.dataset.themeId === 'linen' && Array.from(document.querySelectorAll('[role=status]')).some(el=>el.textContent.includes('Cambios sin guardar'))");
    await shot('settings-save-error.png');
    await evaluate('window.__restoreFetch()');
    await evaluate("Array.from(document.querySelectorAll('.q-card__actions button')).find(b=>b.textContent.includes('Cancelar') || b.textContent.includes('Cancel') || b.textContent.includes('Cancelar')).click()");
    await evaluate("Array.from(document.querySelectorAll('.q-notification button')).find(b=>b.getAttribute('aria-label')==='Cerrar')?.click()");
    await evaluate("document.querySelector('[data-preview-id=forest]').click()");
    await evaluate("Array.from(document.querySelectorAll('.q-card__actions button')).find(b=>b.textContent.includes('Guardar preferencias')).click()");
    await waitFor("fetch('/api/v1/users/me').then(r=>r.json()).then(u=>u.preferences.appearance?.theme_id === 'forest')", 'appearance save');
    await navigate('/settings');
    await check('Save and reload restores appearance', "document.documentElement.dataset.themeId === 'forest'");
    // Exercise the store's system watcher and cache isolation through real browser modules.
    await evaluate(`import('/src/stores/theme.ts').then(({useThemeStore}) => { const s=useThemeStore(); s.preview({...s.appearance, mode:'system'}); })`);
    await call('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-color-scheme', value: 'dark' }] });
    await waitFor("document.documentElement.dataset.theme === 'dark'", 'system dark update');
    await call('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-color-scheme', value: 'light' }] });
    await waitFor("document.documentElement.dataset.theme === 'light'", 'system light update');
    checks.push('System mode follows live OS changes');
    await check('Corrupt cache uses defaults', `import('/src/utils/appearance.ts').then(m=>{const key=m.appearanceKey('test-corrupt');localStorage.setItem(key,'broken');const value=m.readAppearance(key);localStorage.removeItem(key);return value.mode==='system' && value.theme_id==='orbit';})`);
    await check('Account cache does not leak to anonymous scope', `import('/src/stores/theme.ts').then(async ({useThemeStore})=>{const {useAuthStore}=await import('/src/stores/auth.ts');const s=useThemeStore();const id=useAuthStore().user.id;s.bindAccount('test-other-account');const isolated=s.appearance.theme_id==='orbit';s.bindAccount(null);const anonymous=s.appearance.theme_id==='orbit';s.bindAccount(id);return isolated && anonymous;})`);
    await evaluate("import('/src/stores/theme.ts').then(({useThemeStore})=>useThemeStore().cancelPreview())");
    await navigate('/tasks');
    await check('Table has explicit record actions', "document.querySelectorAll('button[aria-label=\"Abrir registro\"]').length > 0");
    await check('Table retains create action', "Array.from(document.querySelectorAll('.dt-actions button')).some(b=>b.textContent.includes('Nuevo'))");
    await evaluate("document.querySelector('.q-table tbody [role=checkbox]').click()");
    await check('Selection keeps create and contextual actions', "Array.from(document.querySelectorAll('.dt-actions button')).some(b=>b.textContent.includes('Nuevo')) && Array.from(document.querySelectorAll('.dt-actions button')).some(b=>b.textContent.includes('Editar'))");
    await evaluate("document.querySelector('.q-table tbody [role=checkbox]').click()");
    await check('Failed table preference is not confirmed', `import('/src/stores/datatable.ts').then(async ({useDataTableStore})=>{
      const save=window.fetch; const store=useDataTableStore();
      window.fetch=(input,init)=>String(input).includes('/datatable/ui-failure-check')?Promise.resolve(new Response(JSON.stringify({error:'simulated failure',type:'internal'}),{status:500,headers:{'Content-Type':'application/json'}})):save(input,init);
      let rejected=false;try {await store.saveVisibleColumns('ui-failure-check',['title']);} catch {rejected=true;} finally {window.fetch=save;}
      return rejected && !store.tables['ui-failure-check'];
    })`);
    await evaluate("document.querySelector('button[aria-label=\"Abrir registro\"]').click()");
    await waitFor("location.pathname.startsWith('/tasks/') && !!document.querySelector('.detail-page')", 'open task');
    checks.push('Single click opens task detail');
    await navigate('/board');
    await check('Board offers non-drag move', "document.querySelectorAll('button[aria-label=\"Mover a…\"]').length > 0");
    const beforeMove = await evaluate("document.querySelector('.kanban-column[data-status=todo] .task-title').textContent");
    await evaluate(`(() => {const originalFetch=window.fetch;window.__restoreFetch=()=>{window.fetch=originalFetch;};window.fetch=(input,init)=>String(input).includes('/api/v1/tasks/') && init?.method==='PATCH'?Promise.resolve(new Response(JSON.stringify({error:'simulated failure',type:'internal'}),{status:500,headers:{'Content-Type':'application/json'}})):originalFetch(input,init);document.querySelector('.kanban-column[data-status=todo] button[aria-label="Mover a…"]').click();})()`);
    await waitFor("!!document.querySelector('.q-menu .q-item')", 'move menu');
    await evaluate("Array.from(document.querySelectorAll('.q-menu .q-item')).find(el=>el.textContent.includes('En progreso')).click()");
    await waitFor("Array.from(document.querySelectorAll('.q-notification')).some(el=>el.textContent.includes('mover')) && !document.querySelector('.kanban-task-card[aria-busy=true]')", 'move completion');
    await check('Failed move restores the original column', `Array.from(document.querySelectorAll('.kanban-column[data-status=todo] .task-title')).some(el=>el.textContent===${JSON.stringify(beforeMove)})`);
    await evaluate('window.__restoreFetch()');
    movedTaskId = await evaluate("document.querySelector('.kanban-column[data-status=todo] a[href^=\"/tasks/\"]').getAttribute('href').split('/').pop()");
    await evaluate("document.querySelector('.kanban-column[data-status=todo] button[aria-label=\"Mover a…\"]').click()");
    await waitFor("!!document.querySelector('.q-menu .q-item')", 'successful move menu');
    await evaluate("Array.from(document.querySelectorAll('.q-menu .q-item')).find(el=>el.textContent.includes('En progreso')).click()");
    await waitFor(`!!document.querySelector('.kanban-column[data-status=in_progress] a[href="/tasks/${movedTaskId}"]') && !document.querySelector('.kanban-task-card[aria-busy=true]')`, 'successful move');
    await check('Move menu persists real task status', `fetch('/api/v1/tasks/${movedTaskId}').then(r=>r.json()).then(task=>task.status==='in_progress')`);
    await evaluate(`fetch('/api/v1/tasks/${movedTaskId}',{method:'PATCH',headers:{'Content-Type':'application/json'},body:JSON.stringify({status:'todo'})}).then(r=>{if(!r.ok)throw Error('Could not restore task status');})`);
    movedTaskId = undefined;
    await call('Emulation.setDeviceMetricsOverride', { width: 320, height: 800, deviceScaleFactor: 1, mobile: true });
    await navigate('/board');
    await check('Mobile board shows one state', "document.querySelectorAll('.kanban-column').length === 1 && document.documentElement.scrollWidth <= innerWidth");
    await navigate('/tasks/new');
    await check('320px form has no page overflow', 'document.documentElement.scrollWidth <= innerWidth');
    await call('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Tab', code: 'Tab', windowsVirtualKeyCode: 9 });
    await call('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Tab', code: 'Tab', windowsVirtualKeyCode: 9 });
    await check('Keyboard can focus a control', "document.activeElement !== document.body && ['A','BUTTON','INPUT'].includes(document.activeElement.tagName)");
    await check('Viewport permits zoom', "!document.querySelector('meta[name=viewport]').content.includes('user-scalable=no')");
    // 200% desktop zoom halves the CSS viewport while retaining physical pixels.
    await call('Emulation.setDeviceMetricsOverride', { width: 720, height: 500, deviceScaleFactor: 2, mobile: false });
    await navigate('/tasks/new');
    await check('200% desktop reflow keeps form actions', "document.documentElement.scrollWidth <= innerWidth && Array.from(document.querySelectorAll('.form-actions button')).some(b=>b.textContent.includes('Guardar'))");
    await shot('form-200-percent-reflow.png');
    await call('Emulation.setDeviceMetricsOverride', { width: 1440, height: 1000, deviceScaleFactor: 1, mobile: false });
    await navigate('/tasks');
    for (const theme_id of ['orbit', 'linen', 'forest']) {
      for (const mode of ['light', 'dark']) {
        for (const density of ['comfortable', 'compact']) {
          await evaluate(`import('/src/stores/theme.ts').then(({useThemeStore})=>useThemeStore().preview({theme_id:${JSON.stringify(theme_id)},mode:${JSON.stringify(mode)},density:${JSON.stringify(density)},reduced_motion:true}))`);
          await check(`Contrast ${theme_id}/${mode}/${density}`, `(() => {
            const style=getComputedStyle(document.documentElement);
            const luminance=(hex)=>{const channels=hex.trim().replace('#','').match(/../g).map(x=>parseInt(x,16)/255).map(v=>v<=.04045?v/12.92:((v+.055)/1.055)**2.4);return channels[0]*.2126+channels[1]*.7152+channels[2]*.0722;};
            const ratio=(a,b)=>{const l1=luminance(a),l2=luminance(b);return (Math.max(l1,l2)+.05)/(Math.min(l1,l2)+.05);};
            const css=name=>style.getPropertyValue(name);
            return ['--surface-page','--surface-panel','--surface-nav'].every(bg=>['--text-primary','--text-secondary','--text-accent'].every(text=>ratio(css(text),css(bg))>=4.5)) && ratio(css('--action-bg'),'#ffffff')>=4.5;
          })()`);
          await shot(`table-${theme_id}-${mode}-${density}.png`);
        }
      }
    }
    await evaluate("import('/src/stores/theme.ts').then(({useThemeStore})=>useThemeStore().cancelPreview())");
    for (const width of [320,360,768]) {
      await call('Emulation.setDeviceMetricsOverride', { width, height: 900, deviceScaleFactor: 1, mobile: width < 600 });
      await navigate('/tasks');
      await check(`List reflow ${width}px`, 'document.documentElement.scrollWidth <= innerWidth');
      await shot(`table-${width}.png`);
    }

  } finally {
    await evaluate('window.__restoreFetch?.()');
    if (movedTaskId) await evaluate(`fetch('/api/v1/tasks/${movedTaskId}',{method:'PATCH',headers:{'Content-Type':'application/json'},body:JSON.stringify({status:'todo'})}).then(r=>{if(!r.ok)throw Error('Could not restore task status');})`);
    await evaluate(`fetch('/api/v1/users/me/preferences', { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(${JSON.stringify({ theme: original.theme, appearance: original.appearance ?? { theme_id: 'orbit', density: 'comfortable', reduced_motion: false } })}) }).then(r=>{ if (!r.ok) throw Error('Could not restore test appearance'); })`);
    await call('Emulation.setEmulatedMedia', { features: [] });
    await call('Emulation.setDeviceMetricsOverride', { width: 1440, height: 1000, deviceScaleFactor: 1, mobile: false });
  }
  await writeFile(join(output, 'verification.json'), JSON.stringify({ checkedAt: new Date().toISOString(), checks }, null, 2));
}
