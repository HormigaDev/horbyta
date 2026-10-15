import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
const require = createRequire(new URL('../frontend/package.json', import.meta.url));
const ts = require('typescript');
const source = await readFile(new URL('../frontend/src/utils/attachmentPreview.ts', import.meta.url), 'utf8');
const code = ts.transpileModule(source, {compilerOptions:{module:ts.ModuleKind.ESNext}}).outputText;
const {previewKind,decodePreviewText,attachmentIdFromUrl,registerPreviewAttachments,getPreviewAttachment} = await import(`data:text/javascript;base64,${Buffer.from(code).toString('base64')}`);
const cases=[['criteria.md','text/plain','markdown'],['criteria.markdown','application/octet-stream','markdown'],['README','text/markdown; charset=utf-8','markdown'],['photo.png','image/png','image'],['reference.svg','image/svg+xml','image'],['voice.ogg','audio/ogg','audio'],['voice.mp3','application/octet-stream','audio'],['demo.mp4','video/mp4','video'],['demo.webm','application/octet-stream','video'],['contract.pdf','application/pdf','pdf'],['archive.zip','application/zip','unsupported'],['fake.md','text/html','text'],['fake.pdf','text/html','text'],['page.html','text/plain','text']];
cases.push(['notes.txt','text/plain','text'],['config.json','application/json','text'],['layout.xml','application/xml','text'],['script.ts','application/octet-stream','text'],['README','application/octet-stream','text'],['contract.docx','application/vnd.openxmlformats-officedocument.wordprocessingml.document','unsupported'],['binary.exe','text/plain','unsupported']);
const utf8=new TextEncoder().encode('Texto <script>seguro</script>');
assert.equal(decodePreviewText(utf8.buffer),'Texto <script>seguro</script>');
assert.equal(decodePreviewText(new Uint8Array([0,1,2]).buffer),null);
assert.equal(decodePreviewText(new Uint8Array([255,254,65,0]).buffer),'A');
assert.equal(decodePreviewText(new Uint8Array([195,40]).buffer),null);
for(const [name,mime,kind] of cases)assert.equal(previewKind(name,mime),kind,`${name}: ${mime}`);
globalThis.window={location:{origin:'https://horbyta.test'}};
const id='00000000-0000-4000-8000-000000000001';
assert.equal(attachmentIdFromUrl(`/api/v1/attachments/${id}/raw`),id);
assert.equal(attachmentIdFromUrl(`https://external.test/api/v1/attachments/${id}/raw`),null);
assert.equal(attachmentIdFromUrl('javascript:alert(1)'),null);
assert.equal(attachmentIdFromUrl('/api/v1/attachments/invalid/raw'),null);
registerPreviewAttachments([{id,original_name:'criteria.md',mime_type:'text/markdown'}]);
registerPreviewAttachments([{id,original_name:'criteria.md'}]);
assert.equal(getPreviewAttachment(id).mime_type,'text/markdown');
console.log(`${cases.length+5} comprobaciones de formato, MIME y enlaces de adjuntos: OK`);

const linksSource = await readFile(new URL('../frontend/src/utils/messageLinks.ts', import.meta.url), 'utf8');
const linksCode = ts.transpileModule(linksSource, {compilerOptions:{module:ts.ModuleKind.ESNext,target:ts.ScriptTarget.ES2022}}).outputText;
const {messageLinks,conversationMessageParts} = await import(`data:text/javascript;base64,${Buffer.from(linksCode).toString('base64')}`);
for (const [text, expected] of [
  ['Sin enlaces', []],
  ['(https://example.com/docs).', ['https://example.com/docs']],
  ['https://example.com/a_(b)', ['https://example.com/a_(b)']],
  ['javascript:alert(1) <script>alert(1)</script>', []],
  ['https://example.com/a y http://example.org/b!', ['https://example.com/a','http://example.org/b']],
  ['https://', []],
]) {
  const parts = messageLinks(text);
  assert.equal(parts.map(part=>part.text).join(''), text);
  assert.deepEqual(parts.filter(part=>part.href).map(part=>part.href), expected);
}
console.log('6 comprobaciones de enlaces de mensajes y texto seguro: OK');

const navigationSource = await readFile(new URL('../frontend/src/utils/notificationNavigation.ts', import.meta.url), 'utf8');
const navigationCode = ts.transpileModule(navigationSource, {compilerOptions:{module:ts.ModuleKind.ESNext,target:ts.ScriptTarget.ES2022}}).outputText;
const {scopeNotificationTarget,notificationPreview} = await import(`data:text/javascript;base64,${Buffer.from(navigationCode).toString('base64')}`);
const payload = {project_id:id,revision:2,message_id:id};
assert.equal(scopeNotificationTarget(payload).message,id);
assert.equal(scopeNotificationTarget({...payload,change_request_id:id}).change,id);
assert.equal(scopeNotificationTarget(payload).change,null);
assert.equal(scopeNotificationTarget({project_id:id,revision:2}).revision,2);
assert.equal(scopeNotificationTarget({project_id:'invalid',revision:2}),null);
assert.equal(scopeNotificationTarget({project_id:id,revision:'2'}),null);
assert.equal(notificationPreview({payload,body:'Un mensaje corto'}),'Un mensaje corto');
assert.equal(notificationPreview({payload,body:'x'.repeat(61)}),'');
assert.equal(notificationPreview({payload,body:'😀'.repeat(60)}),'😀'.repeat(60));
assert.equal(notificationPreview({payload,body:'😀'.repeat(61)}),'');
assert.equal(notificationPreview({payload,body:'  uno\n dos  '}),'uno dos');
assert.equal(Array.from(notificationPreview({payload:null,body:'x'.repeat(1000)})).length,178);
console.log('12 comprobaciones de navegación y previews de notificaciones: OK');

const ref={kind:'project',id,label:'Proyecto',start:2,end:11};
assert.equal(conversationMessageParts('😀 @Proyecto listo',[ref]).find(p=>p.resource)?.text,'@Proyecto');
assert.equal(conversationMessageParts('😀 @Proyecto listo',[ref]).map(p=>p.text).join(''),'😀 @Proyecto listo');
assert.equal(conversationMessageParts('😀 @Proyecto listo',[{...ref,start:1}]).some(p=>p.resource),false);
assert.equal(conversationMessageParts('Texto',[{kind:'project',id,label:'Anterior'}]).at(-1).resource.label,'Anterior');
assert.equal(conversationMessageParts('@Proyecto https://example.com',[{...ref,start:0,end:9}]).filter(p=>p.href).length,1);
assert.equal(conversationMessageParts('@Proyecto @Proyecto',[{...ref,start:0,end:9},{...ref,id:'other',start:10,end:19}]).filter(p=>p.resource).length,2);
console.log('6 comprobaciones de menciones inline, posiciones Unicode y referencias anteriores: OK');
