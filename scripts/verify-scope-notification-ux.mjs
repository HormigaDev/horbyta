import assert from 'node:assert/strict';
import {readFile, writeFile} from 'node:fs/promises';
import {join} from 'node:path';
export async function verifyScopeNotificationUx({evaluate,navigate,waitFor,output,account,call}) {
  const state=JSON.parse(await readFile('target/scope-tests/state.json','utf8'));
  const project=state.conversation_project;
  const api=async(path,body)=>{
    const result=await evaluate(`fetch('/api/v1'+${JSON.stringify(path)},{credentials:'include',${body?`method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(${JSON.stringify(body)}),`:''}}).then(async r=>({status:r.status,body:await r.json()}))`);
    assert.equal(result.status,200,JSON.stringify(result.body));return result.body;
  };
  assert.equal((await api('/users/me')).email,'scope-tests@horbyta.test');
  await call('Emulation.setFocusEmulationEnabled',{enabled:true});
  await navigate(`/projects/${project}`);
  await waitFor("!!document.querySelector('.scope-tabs')",'Alcance preparado');
  const center=async()=>{
    await evaluate("document.querySelector('button[aria-label=\"Notificaciones\"]').click()");
    await waitFor("!!document.querySelector('.notifications-list-item')",'Centro de notificaciones');
  };
  await center();
  await waitFor("Array.from(document.querySelectorAll('.notifications-list-item')).some(n=>n.textContent.includes('Aviso anterior'))",'Aviso legado');
  await evaluate("Array.from(document.querySelectorAll('.notifications-list-item')).find(n=>n.textContent.includes('Aviso anterior')).click()");
  await waitFor("document.querySelector('.scope-message-target')?.textContent.includes('nota.md')",'Aviso anterior apunta a su mensaje');
  assert.equal(await evaluate("!!document.querySelector('.notification-detail-body')"),false);
  assert.equal(await evaluate("Array.from(document.querySelectorAll('.q-dialog')).some(d=>d.textContent.includes('Detalles del error'))"),false);
  await api('/auth/login-client',{user_id:state.owner,email:state.client_email,password:account.password});
  await evaluate("Array.from(document.querySelectorAll('.scope-tabs .q-tab')).find(t=>t.textContent.includes('Alcance')).click()");
  const long='Mensaje largo '+ 'x'.repeat(3500);
  const result=await api(`/scope/client/projects/${project}/feedback`,{revision:1,kind:'comment',text:long,attachment_ids:[]});
  const message=result.events.find(e=>e.text===long);
  assert(message);
  await waitFor("!!document.querySelector('.q-notification')",'Aviso mensaje largo');
  assert.equal(await evaluate("Array.from(document.querySelectorAll('.q-notification__caption')).some(c=>c.textContent.length>60)"),false);
  await center();
  assert.equal(await evaluate(`document.querySelector('.notifications-list-item').textContent.includes(${JSON.stringify(long.slice(0,80))})`),false);
  await evaluate("document.querySelector('.notifications-list-item').click()");
  await waitFor(`document.querySelector('.scope-message-target')?.dataset.messageId===${JSON.stringify(message.id)}`,'Aviso nuevo apunta al ID exacto');
  assert.equal(await evaluate("!!document.querySelector('.notification-detail-body')"),false);
  assert(await evaluate("document.querySelectorAll('.scope-messages .q-message').length<=100"));
  const older=await api(`/scope/projects/${project}/conversation/1`);
  const page=await api(`/scope/projects/${project}/conversation/1?before=${older.items[0].id}`);
  const old=page.items[25];
  await navigate(`/projects/${project}?conversation=1&revision=1&message=${old.id}`);
  await waitFor(`document.querySelector('.scope-message-target')?.dataset.messageId===${JSON.stringify(old.id)}`,'Mensaje fuera de últimos 100 localizado');
  assert(await evaluate("document.querySelectorAll('.scope-messages .q-message').length<=100"));
  await call('Page.captureScreenshot',{format:'png',captureBeyondViewport:false}).then(r=>writeFile(join(output,'scope-notification-target.png'),Buffer.from(r.data,'base64')));
  await writeFile(join(output,'scope-notification-ux.json'),JSON.stringify({checks:['Avisos anteriores abren mensaje sin modal de error','Mensaje largo omitido en toast y listado','Aviso nuevo enlaza ID exacto','Carga alrededor de mensaje antiguo sin superar 100 activos']},null,2));
  console.log('Notificaciones de conversación: 4 comprobaciones UX OK');
}
