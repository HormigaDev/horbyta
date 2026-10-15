import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

// Revisión de lectura del seed: no crea proyectos ni altera acuerdos/finanzas.
export async function verifyDevelopmentSeed({call,evaluate,navigate,waitFor,output}) {
  const report=JSON.parse(await readFile('target/development-seed/report.json','utf8'));
  const checks=[];
  const assert=(ok,label)=>{if(!ok)throw Error(label);checks.push(label);};
  const api=async(path,method='GET',body)=>{
    const result=await evaluate(`fetch('/api/v1'+${JSON.stringify(path)},{method:${JSON.stringify(method)},credentials:'include',headers:{'Content-Type':'application/json'},${body===undefined?'':`body:JSON.stringify(${JSON.stringify(body)}),`}}).then(async r=>({status:r.status,body:await r.json()}))`);
    if(result.status!==200)throw Error(`Seed: ${method} ${path} respondió ${result.status}`);
    return result.body;
  };
  const shot=async(name)=>{
    await evaluate('new Promise(resolve=>setTimeout(resolve,500))');
    const {data}=await call('Page.captureScreenshot',{format:'png',captureBeyondViewport:true});
    await writeFile(join(output,`seed-${name}.png`),Buffer.from(data,'base64'));
  };
  const user=await api('/users/me');
  assert(user.id===report.admin_id&&user.email==='admin@horbyta.test'&&user.role==='admin','Administrador sembrado accede por el formulario real');
  const q={page:1,limit:50,order_by:null,order_asc:true,filters:[]};
  const lists={};
  for(const [entity,total] of Object.entries({projects:12,clients:8,tasks:143,invoices:28,payments:20})) {
    lists[entity]=await api(`/${entity}/search?q=${encodeURIComponent(JSON.stringify(q))}`);
    assert(lists[entity].total===total,`${entity}: datos visibles de la cuenta, total ${total}`);
  }
  const project=lists.projects.items.find(p=>p.name==='Portal editorial Lumen');
  assert(!!project,'Proyecto documental preparado disponible');
  await navigate(`/projects/${project.id}`);
  await waitFor("document.querySelector('.scope-document').textContent.includes('Acuerdo activo')",'acuerdo sembrado');
  const scope=await api(`/scope/projects/${project.id}`);
  assert(scope.accepted_revision===2&&scope.revisions[0].document.documents.length===3,'Acuerdo versionado con principal, requisito separado y referencia');
  assert(scope.revisions[0].document.documents.some(d=>d.role==='annex')&&scope.revisions[0].document.documents.some(d=>d.role==='reference'),'Documentos muestran su función en el alcance');
  for(const file of scope.revisions[0].attachment_snapshot) {
    const asset=await evaluate(`fetch('/api/v1/attachments/'+${JSON.stringify(file.id)}+'/raw',{credentials:'include'}).then(async r=>({status:r.status,bytes:(await r.arrayBuffer()).byteLength}))`);
    assert(asset.status===200&&asset.bytes===file.size_bytes,`Adjunto real descargable: ${file.original_name}`);
  }
  await shot('proyecto-documentos');
  await navigate(`/client/u/${user.id}/login`);
  await waitFor("!!document.querySelector('input[type=email]')",'acceso del cliente sembrado');
  await evaluate("(()=>{for(const [type,value] of [['email','cliente1@horbyta.test'],['password','Cliente@123']]){const input=document.querySelector('input[type='+type+']');input.value=value;input.dispatchEvent(new Event('input',{bubbles:true}));}document.querySelector('form').requestSubmit();})()");
  await waitFor("location.pathname==='/client'&&!!document.querySelector('.q-page')",'portal sembrado autenticado');
  assert((await api('/auth/client-me')).user_id===user.id,'Cliente ficticio accede por el formulario del portal');
  await navigate(`/client/projects/${project.id}`);
  assert(await evaluate("document.querySelector('.scope-document').textContent.includes('Acceso y validación de formularios')&&!!document.querySelector('nav[aria-label=\"Índice de documentos del alcance\"]')"),'Portal lee requisitos separados y su índice');
  await shot('portal-documentos');
  await call('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:true});
  await waitFor("document.querySelector('.scope-document').getBoundingClientRect().width>300",'portal móvil estable');
  assert(await evaluate("document.documentElement.scrollWidth<=innerWidth+2"),'Portal documental del seed legible en móvil');
  await shot('portal-movil');
  await call('Emulation.setDeviceMetricsOverride',{width:1440,height:1000,deviceScaleFactor:1,mobile:false});
  await api('/auth/logout-client','POST');
  await navigate('/');
  await writeFile(join(output,'seed-verification.json'),JSON.stringify({checkedAt:new Date().toISOString(),checks},null,2));
  checks.forEach(label=>console.log('Verificado seed: '+label));
}
