import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';

export async function verifyForms({ evaluate, navigate, waitFor, output }) {
  const checks = [];
  let createdTaskId = null;
  if (!await evaluate("fetch('/api/v1/users/me').then(r=>r.json()).then(u=>u.email==='ui-tests@horbyta.test')")) throw Error('Form verification requires the dedicated test account.');
  const click = text => evaluate(`Array.from(document.querySelectorAll('.q-dialog button')).find(b=>b.querySelector('.block')?.textContent===${JSON.stringify(text)}).click()`);
  await navigate('/tasks/new');
  try {
    await evaluate("(()=>{const input=document.querySelector('.form-page input');input.value='[UI] Borrador de comprobación';input.dispatchEvent(new Event('input',{bubbles:true}));})()");
    await waitFor("Object.keys(localStorage).some(k=>k.startsWith('form-draft:') && localStorage.getItem(k).includes('[UI] Borrador de comprobación'))", 'draft autosave');
    await evaluate("document.querySelector('a[href=\"/tasks\"]').click()");
    await waitFor("Array.from(document.querySelectorAll('.q-dialog')).some(el=>el.textContent.includes('Cambios sin guardar'))", 'unsaved changes guard');
    await click('Quedarme');
    await waitFor("!document.querySelector('.q-dialog')", 'stay on form');
    if (!await evaluate("document.querySelector('.form-page input').value === '[UI] Borrador de comprobación'")) throw Error('Cancel navigation lost the form.');
    checks.push('Cancel navigation preserves unsaved values');
    await evaluate("document.querySelector('a[href=\"/tasks\"]').click()");
    await waitFor("!!document.querySelector('.q-dialog')", 'leave confirmation');
    await click('Salir de todos modos');
    await waitFor("location.pathname === '/tasks'", 'leave form');
    await navigate('/tasks/new');
    await waitFor("Array.from(document.querySelectorAll('.q-dialog')).some(el=>el.textContent.includes('Borrador guardado'))", 'draft restore dialog');
    await click('Restaurar borrador');
    await waitFor("document.querySelector('.form-page input')?.value==='[UI] Borrador de comprobación'", 'restored draft');
    checks.push('Draft survives route leave and restores');
    await evaluate("(()=>{const input=document.querySelector('.form-page input');input.value='';input.dispatchEvent(new Event('input',{bubbles:true}));for(const key of Object.keys(localStorage)){if(key.startsWith('form-draft:')&&key.endsWith(':task:new'))localStorage.removeItem(key);}})()");
    await evaluate("document.querySelector('button[aria-label=\"Preferencias de formulario\"]').click()");
    await waitFor("Array.from(document.querySelectorAll('.q-dialog button')).some(b=>b.textContent.includes('Guardar actual'))", 'preset dialog');
    await click('Guardar actual');
    await waitFor("!!document.querySelector('.q-dialog input[type=text]')", 'preset name');
    await evaluate("(()=>{const input=document.querySelector('.q-dialog input[type=text]');input.value='UI preset failure';input.dispatchEvent(new Event('input',{bubbles:true}));const savedFetch=window.fetch;window.__restoreFetch=()=>{window.fetch=savedFetch;};window.fetch=(url,init)=>String(url).includes('/form_pref/task')&&init?.method==='PUT'?Promise.resolve(new Response(JSON.stringify({error:'simulated failure',type:'internal'}),{status:500,headers:{'Content-Type':'application/json'}})):savedFetch(url,init);})()");
    await click('Guardar');
    await waitFor("Array.from(document.querySelectorAll('.q-notification')).some(el=>el.textContent.includes('preset'))", 'preset failure');
    if (!await evaluate("document.querySelector('.q-dialog input[type=text]')?.value==='UI preset failure'")) throw Error('Failed preset save closed the retry dialog.');
    checks.push('Failed preset stays editable and does not claim success');
    await evaluate('window.__restoreFetch()');
    await click('Cancelar');
    await click('Cerrar');
    await evaluate("(()=>{const input=document.querySelector('.form-page input');input.value='[UI] Temporary create/edit verification';input.dispatchEvent(new Event('input',{bubbles:true}));document.querySelector('.form-actions button:last-child').click();})()");
    await waitFor("location.pathname.startsWith('/tasks/') && !location.pathname.endsWith('/edit') && !!document.querySelector('.detail-page')", 'create task from form');
    createdTaskId = await evaluate('location.pathname.split("/").pop()');
    await navigate(`/tasks/${createdTaskId}/edit`);
    await evaluate("(()=>{const input=document.querySelector('.form-page input');input.value='[UI] Edited verification';input.dispatchEvent(new Event('input',{bubbles:true}));document.querySelector('.form-actions button:last-child').click();})()");
    await waitFor("location.pathname.startsWith('/tasks/') && !location.pathname.endsWith('/edit') && !!document.querySelector('.detail-page')", 'edit task from form');
    if (!await evaluate(`fetch('/api/v1/tasks/${createdTaskId}').then(r=>r.json()).then(task=>task.title==='[UI] Edited verification')`)) throw Error('Edited task was not persisted.');
    checks.push('Task creates and edits through the real form/API');
    await navigate('/projects/new');
    if (!await evaluate('!!document.querySelector(".form-page .form-actions")')) throw Error('Project form lacks shared actions.');
    checks.push('Project pilot uses shared form hierarchy');
    await navigate('/invoices/new');
    await navigate('/payments/new');
    await navigate('/change-requests/new');
    checks.push('Invoice, payment and change-request forms render');
  } finally {
    await evaluate('window.__restoreFetch?.()');
    if (createdTaskId) await evaluate(`fetch('/api/v1/tasks/${createdTaskId}', { method:'DELETE' }).then(r=>{if(!r.ok)throw Error('Could not remove temporary test task');})`);
  }
  await writeFile(join(output, 'form-verification.json'), JSON.stringify({ checkedAt: new Date().toISOString(), checks }, null, 2));
  for (const label of checks) console.log(`Verified: ${label}`);
}
