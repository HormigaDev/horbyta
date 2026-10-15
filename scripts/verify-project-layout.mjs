import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
export async function verifyProjectLayout({ evaluate, navigate, waitFor, output, account, call }) {
  const state = JSON.parse(await readFile('target/scope-tests/state.json', 'utf8'));
  const checks = [];
  const capture = async (name) => {
    await delay(250);
    const { data } = await call('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
    await writeFile(join(output, name), Buffer.from(data, 'base64'));
  };
  const tab = (name) => evaluate(`Array.from(document.querySelectorAll('.scope-tabs .q-tab')).find(t=>t.textContent.includes(${JSON.stringify(name)})).click()`);
  for (const client of [false, true]) {
    if (client) {
      await evaluate(`(async()=>{const r=await fetch('/api/v1/auth/login-client',{method:'POST',credentials:'include',headers:{'Content-Type':'application/json'},body:JSON.stringify(${JSON.stringify({user_id:state.owner,email:state.client_email,password:account.password})})});if(!r.ok)throw Error('Client login failed');})()`);
    }
    await call('Emulation.setDeviceMetricsOverride', { width:1440, height:1000, deviceScaleFactor:1, mobile:false });
    await navigate(`${client ? '/client' : ''}/projects/${state.project}`);
    await waitFor("document.querySelector('.scope-tabs .q-tab--active')?.textContent.includes('Resumen')&&!!document.querySelector('.project-overview')", 'Overview');
    assert(!(await evaluate("!!document.querySelector('.scope-revision-toolbar')")));
    await capture(`project-overview-${client ? 'client' : 'professional'}.png`);
    await tab('Alcance');
    await waitFor("!!document.querySelector('.scope-revision-toolbar')", 'Scope');
    assert.equal(await evaluate("document.querySelectorAll('.scope-revision-toolbar .q-select').length"),1);
    assert(await evaluate("Array.from(document.querySelectorAll('.scope-section__body')).every(e=>!e.textContent.includes('200.0000'))"));
    await capture(`project-scope-${client ? 'client' : 'professional'}.png`);
    // Inspect the real dark theme tokens without persisting a test preference.
    await evaluate("(()=>{const theme=document.getElementById('q-app').__vue_app__.config.globalProperties.$pinia._s.get('theme');theme.preview({...theme.appearance,mode:'dark'});})()");
    await capture(`project-scope-dark-${client ? 'client' : 'professional'}.png`);
    await evaluate("document.getElementById('q-app').__vue_app__.config.globalProperties.$pinia._s.get('theme').cancelPreview()");
    await tab('Conversación');
    await waitFor("!!document.querySelector('.change-chat-window')", 'Conversation');
    assert(!(await evaluate("!!document.querySelector('.scope-revision-toolbar')")));
    await evaluate(`document.querySelector(${JSON.stringify('.project-section-index a[href="#scope-project-'+state.project+'-deliverables"]')}).click()`);
    await waitFor(`document.querySelector('.scope-tabs .q-tab--active')?.textContent.includes('Alcance')&&document.activeElement?.id==='scope-project-${state.project}-deliverables'`, 'Index');
    await call('Emulation.setDeviceMetricsOverride', {width:390,height:844,deviceScaleFactor:1,mobile:true});
    await navigate(`${client ? '/client' : ''}/projects/${state.project}`);
    await waitFor("!!document.querySelector('.project-overview')", 'Mobile overview');
    assert(await evaluate('document.documentElement.scrollWidth<=innerWidth+1'), 'Overview mobile overflow');
    await capture(`project-overview-mobile-${client ? 'client' : 'professional'}.png`);
    await tab('Alcance');
    assert(await evaluate('document.documentElement.scrollWidth<=innerWidth+1'), 'Scope mobile overflow');
    await capture(`project-scope-mobile-${client ? 'client' : 'professional'}.png`);
    checks.push(`${client ? 'Cliente' : 'Profesional'}: Resumen, Alcance, Conversación, índice, temas y móvil`);
  }
  await writeFile(join(output, 'project-layout-verification.json'), JSON.stringify({ checks }, null, 2));
  console.log('Diseño del proyecto verificado en ambos roles, temas y móvil: OK');
}
