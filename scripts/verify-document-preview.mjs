import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
export async function verifyDocumentPreview({evaluate,navigate,waitFor,output,account,call}) {
 const state=JSON.parse(await readFile('target/scope-tests/state.json','utf8'));
 const api=(path,body)=>evaluate(`(async()=>{const r=await fetch('/api/v1'+${JSON.stringify(path)},{method:'POST',credentials:'include',headers:{'Content-Type':'application/json'},body:JSON.stringify(${JSON.stringify(body)})});const v=await r.json();if(!r.ok)throw Error(JSON.stringify(v));return v;})()`);
 const project=await api('/projects',{name:'[Visor] Documentos bajo demanda',description:'Prueba aislada',client_id:state.client,billable:false});
 const docId=await evaluate('crypto.randomUUID()');
 const files=await evaluate(`(async()=>{const f=new FormData();f.append('files',new Blob(['# Criterios\\n\\n**Contenido formateado**'],{type:'text/markdown'}),'criterios.md');f.append('files',new Blob(['<script>window.badPreview=true</script>\\nTexto seguro'],{type:'text/plain'}),'notas.txt');f.append('files',new Blob([Uint8Array.from(atob('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+a++0AAAAASUVORK5CYII='),c=>c.charCodeAt(0))],{type:'image/png'}),'imagen.png');f.append('files',new Blob([new Uint8Array([80,75,3,4])],{type:'application/vnd.openxmlformats-officedocument.wordprocessingml.document'}),'contrato.docx');f.append('files',new Blob(['%PDF-1.4\\n%%EOF'],{type:'application/pdf'}),'propuesta.pdf');const r=await fetch('/api/v1/scope/projects/${project.id}/conversation/files',{method:'POST',credentials:'include',body:f});const v=await r.json();if(!r.ok)throw Error(JSON.stringify(v));return v.items;})()`);
 await api(`/scope/projects/${project.id}`,{expected_revision:0,publish:true,document:{deliverables:'Entregable',criteria:'Verificación',restrictions:'',exclusions:'',requirements:'',precedence:'El documento principal prevalece; los anexos detallan los criterios.',documents:[{id:docId,title:'Documento principal',role:'main',section:null,content:'# Documento reservado\n\n**Lectura en modal**',source_name:null,source_content:null,requirement_id:null}],attachment_ids:files.map(f=>f.id),reference_attachment_ids:[]}});
 const capture=async(name)=>{await delay(250);const {data}=await call('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});await writeFile(join(output,name),Buffer.from(data,'base64'));};
 const openFile=async(name)=>evaluate(`Array.from(document.querySelectorAll('a')).find(a=>a.textContent.includes(${JSON.stringify(name)})).dispatchEvent(new MouseEvent('click',{bubbles:true,cancelable:true,view:window}))`);
 for(const client of [false,true]) {
  if(client)await api('/auth/login-client',{user_id:state.owner,email:state.client_email,password:account.password});
  await navigate(`${client?'/client':''}/projects/${project.id}?tab=scope`);
  await waitFor("!!document.querySelector('.scope-document-index a')",'Lista documental');
  assert(!(await evaluate("document.querySelector('.scope-tab-panels').textContent.includes('Lectura en modal')")),'No se renderiza contenido de documentos dentro de la página');
  await evaluate("document.querySelector('.scope-document-index a').click()");
  await waitFor("document.querySelector('.scope-content-document .markdown-preview h1')?.textContent==='Documento reservado'",'Markdown documental en modal');
  assert(await evaluate("document.querySelector('.scope-content-document strong')?.textContent==='Lectura en modal'"));
  await capture(`document-modal-${client?'client':'professional'}.png`);
  await evaluate("document.querySelector('.scope-content-document button[aria-label=\"Cerrar\"]').click()");
  await waitFor("!document.querySelector('.scope-content-document')",'Cerrar documento');
  await openFile('criterios.md');
  await waitFor("document.querySelector('.attachment-preview-dialog .markdown-preview strong')?.textContent==='Contenido formateado'",'Markdown adjunto');
  await evaluate("document.querySelector('.attachment-preview-dialog button[aria-label=\"Cerrar\"]').click()");
  await waitFor("!document.querySelector('.attachment-preview-dialog')",'Cerrar adjunto');
  await openFile('notas.txt');
  await waitFor("document.querySelector('.attachment-preview-text')?.textContent.includes('Texto seguro')",'Texto escapado');
  assert(!(await evaluate('window.badPreview===true')),'HTML se muestra como texto, sin ejecutarlo');
  await capture(`text-modal-${client?'client':'professional'}.png`);
  await evaluate("document.querySelector('.attachment-preview-dialog button[aria-label=\"Cerrar\"]').click()");
  await waitFor("!document.querySelector('.attachment-preview-dialog')",'Cerrar texto');
  await openFile('imagen.png');
  await waitFor("document.querySelector('.attachment-preview-image')?.naturalWidth>0",'Imagen encuadrada');
  await capture(`image-modal-${client?'client':'professional'}.png`);
  await evaluate("document.querySelector('.attachment-preview-dialog button[aria-label=\"Cerrar\"]').click()");
  await waitFor("!document.querySelector('.attachment-preview-dialog')",'Cerrar imagen');
  // Intercepta la descarga para no guardar binarios de prueba en la laptop.
  await evaluate("window.previewDownload=null;window.originalAnchorClick=HTMLAnchorElement.prototype.click;HTMLAnchorElement.prototype.click=function(){window.previewDownload={url:this.href,name:this.download};}");
  await openFile('contrato.docx');
  assert((await evaluate('window.previewDownload'))?.name==='contrato.docx');
  assert(!(await evaluate("!!document.querySelector('.attachment-preview-dialog')")),'DOCX solo se descarga');
  await evaluate('HTMLAnchorElement.prototype.click=window.originalAnchorClick');
  await evaluate("window.previewPdf=null;window.originalWindowOpen=window.open;window.open=(url,target)=>{window.previewPdf={url,target};return null;}");
  await openFile('propuesta.pdf');
  assert((await evaluate('window.previewPdf'))?.target==='_blank');
  assert(!(await evaluate("!!document.querySelector('.attachment-preview-dialog iframe, .attachment-preview-dialog')")),'PDF sin lector embebido');
  await evaluate('window.open=window.originalWindowOpen');
 }
 console.log('Documentos bajo demanda, Markdown, texto seguro, imagen, PDF y DOCX: OK en ambos roles');
}
