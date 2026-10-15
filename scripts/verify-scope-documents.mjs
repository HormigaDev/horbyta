import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';

// Usa las sesiones y las identidades del esquema aislado del verificador de alcance.
export async function verifyScopeDocuments({api,call,evaluate,navigate,waitFor,click,assert,screenshot,state,output}) {
  const project=await api('/projects','POST',{name:'[Documentos] Negociación Markdown',description:'Laboratorio documental',client_id:state.client,billable:false});
  const task=await api('/tasks','POST',{project_id:project.id,title:'Trabajo bloqueado hasta acuerdo',status:'todo',priority:1,complexity:1,block_parent:true});
  const blocked=await api(`/tasks/${task.id}`,'PATCH',{project_id:project.id,title:task.title,status:'in_progress',priority:1,complexity:1,block_parent:true},409);
  assert(!!blocked,'La ejecución del proyecto nuevo espera el acuerdo');
  await navigate(`/projects/${project.id}`);await click('Preparar alcance');
  const original='# Alcance completo\n\n## Entregables\nTres páginas informativas.\n\n## Criterios\nNavegación verificable.\n\n## Restricciones\nUna revisión.\n\n## Exclusiones\nSin tienda.\n\n## Requisitos\nAcceso autenticado.';
  const path=join(output,'alcance-importado.md');await writeFile(path,original);
  const root=await call('DOM.getDocument',{});const node=await call('DOM.querySelector',{nodeId:root.root.nodeId,selector:'.scope-document form input[type=file][accept]'});
  await call('DOM.setFileInputFiles',{nodeId:node.nodeId,files:[path]});
  await waitFor("document.querySelector('.scope-document').textContent.includes('Importado de alcance-importado.md')",'importación Markdown real');
  await evaluate("(()=>{const input=Array.from(document.querySelectorAll('.scope-document form .q-field')).find(e=>e.textContent.includes('Cómo resolver contradicciones')).querySelector('textarea');input.value='Las exclusiones del principal prevalecen; los anexos detallan los entregables.';input.dispatchEvent(new Event('input',{bubbles:true}));})()");
  await click('Publicar para el cliente');
  try {await waitFor("!Array.from(document.querySelectorAll('button')).some(b=>b.textContent.includes('Publicar para el cliente'))",'publicación documental');}catch(e){await screenshot('documentos-fallo');console.log(await evaluate("document.querySelector('.scope-document').textContent.slice(-2500)"));throw e;}
  let agreement=await api(`/scope/projects/${project.id}`);const doc=agreement.revisions[0].document.documents[0];
  assert(doc.content===original && doc.source_content===original && doc.hash.length===64,'Importación conserva contenido, original y hash sin repartir cinco campos');
  await navigate(`/client/projects/${project.id}`);
  assert(await evaluate("!!document.querySelector('nav[aria-label=\"Índice de documentos del alcance\"]')"),'El cliente tiene índice y lectura de documentos');
  await click('Solicitar ajustes');
  await evaluate("(()=>{const input=document.querySelector('.scope-document form textarea');input.value='Prefiero la propuesta con dos páginas';input.dispatchEvent(new Event('input',{bubbles:true}));})()");
  await click('Solicitar ajustes','.scope-document form button');
  await waitFor("document.querySelector('.scope-document').textContent.includes('Ajustes solicitados')",'ajustes registrados');
  assert(!(await api(`/scope/client/projects/${project.id}`)).active,'Pedir ajustes mantiene bloqueada la ejecución');
  await screenshot('documentos-ajustes');
  await navigate(`/projects/${project.id}`);await click('Usar esta revisión');
  await evaluate("(()=>{const input=Array.from(document.querySelectorAll('.scope-document form textarea')).find(e=>e.value.includes('# Alcance completo'));input.value=input.value.replace('Tres páginas','Dos páginas');input.dispatchEvent(new Event('input',{bubbles:true}));})()");
  await click('Publicar para el cliente');await waitFor("!Array.from(document.querySelectorAll('button')).some(b=>b.textContent.includes('Publicar para el cliente'))",'segunda propuesta');
  agreement=await api(`/scope/projects/${project.id}`);
  assert(agreement.revisions[0].document.documents[0].version===2 && agreement.revisions[0].document.documents[0].source_content===original,'Editar documento crea otra versión y conserva el archivo original');
  await navigate(`/client/projects/${project.id}`);
  await evaluate("Array.from(document.querySelectorAll('.scope-document .q-expansion-item')).find(e=>e.querySelector('.q-item__label')?.textContent.includes('Comparar con otra revisión')).querySelector('.q-item').click()");
  await evaluate("Array.from(document.querySelectorAll('.scope-document .q-expansion-item')).find(e=>e.querySelector('.q-item__label')?.textContent.includes('contenido')).querySelector('.q-item').click()");
  await waitFor("Array.from(document.querySelectorAll('.scope-diff')).some(e=>e.getBoundingClientRect().height>20)",'comparación desplegada');
  assert(await evaluate("Array.from(document.querySelectorAll('.scope-diff')).some(e=>e.textContent.includes('Tres páginas')&&e.textContent.includes('Dos páginas'))"),'Comparación muestra el contenido retirado e incorporado');
  await screenshot('documentos-comparacion');
  await click('Rechazar propuesta');
  await evaluate("(()=>{const input=document.querySelector('.scope-document form textarea');input.value='Deseo recuperar las tres páginas de la primera propuesta';input.dispatchEvent(new Event('input',{bubbles:true}));})()");await click('Rechazar propuesta','.scope-document form button');
  await waitFor("document.querySelector('.scope-document').textContent.includes('Rechazada')",'rechazo inicial');
  const rejected=await api(`/scope/client/projects/${project.id}`);assert(rejected.revisions[0].status==='rejected'&&!rejected.active,'Rechazo inicial conserva motivo e historial sin activar trabajo');
  // La selección de revisión se opera sobre el q-select real.
  await evaluate("document.querySelector('.scope-document .q-select').click()");await click('Revisión 1 — Ajustes solicitados','.q-menu .q-item');
  await click('Solicitar esta versión');await evaluate("(()=>{const input=document.querySelector('.scope-document form textarea');input.value='Confirmo que prefiero la revisión 1';input.dispatchEvent(new Event('input',{bubbles:true}));})()");await click('Solicitar esta versión','.scope-document form button');
  await waitFor("document.querySelector('.scope-document').textContent.includes('Confirmo que prefiero')",'preferencia histórica');
  assert(!(await api(`/scope/client/projects/${project.id}`)).active,'Preferir una revisión antigua solicita una propuesta, no una aceptación');
  await navigate(`/projects/${project.id}`);await evaluate("document.querySelector('.scope-document .q-select').click()");await click('Revisión 1 — Ajustes solicitados','.q-menu .q-item');await click('Usar esta revisión');await click('Publicar para el cliente');await waitFor("!Array.from(document.querySelectorAll('button')).some(b=>b.textContent.includes('Publicar para el cliente'))",'restauración como nueva propuesta');
  agreement=await api(`/scope/projects/${project.id}`);assert(agreement.last_revision===3&&agreement.revisions[0].document.based_on===1&&agreement.revisions[0].document.documents[0].content===original,'Restaurar publica una revisión nueva con su procedencia');
  await navigate(`/client/projects/${project.id}`);await click('Aceptar este alcance');await click('Aceptar este alcance','.q-dialog button');await waitFor("document.querySelector('.scope-document').textContent.includes('Acuerdo activo')",'aceptación final');
  agreement=await api(`/scope/client/projects/${project.id}`);assert(agreement.accepted_revision===3&&!!agreement.revisions[0].accepted_ip&&!!agreement.revisions[0].acceptance_statement,'Aceptación conserva cliente, IP, condiciones y revisión exacta');
  assert((await api(`/scope/tasks/${task.id}`)).allowed,'El acuerdo aceptado habilita su trabajo');
  await evaluate("document.querySelector('.scope-document .q-select').click()");await click('Revisión 1 — Ajustes solicitados','.q-menu .q-item');
  assert(await evaluate("Array.from(document.querySelectorAll('.scope-document .q-badge')).some(e=>e.textContent.includes('Ajustes solicitados'))&&!Array.from(document.querySelectorAll('.scope-document .q-badge')).some(e=>e.textContent.trim()==='Aceptada')"),'La revisión histórica muestra su propio estado y no una aceptación ajena');
  await screenshot('documentos-historial');
  await call('Emulation.setDeviceMetricsOverride', {width:390,height:844,deviceScaleFactor:1,mobile:true});
  await waitFor("document.querySelector('.scope-document').getBoundingClientRect().width > 300",'documentos en layout móvil');
  assert(await evaluate("document.documentElement.scrollWidth <= innerWidth+2 && !!document.querySelector('nav[aria-label=\"Índice de documentos del alcance\"]')"),'Los documentos completos conservan índice y lectura móvil sin desbordamiento');
  await screenshot('documentos-movil');
  await call('Emulation.setDeviceMetricsOverride', {width:1440,height:1000,deviceScaleFactor:1,mobile:false});
  await waitFor("document.querySelector('.scope-document').getBoundingClientRect().width > 600",'documentos en layout de escritorio');
}
