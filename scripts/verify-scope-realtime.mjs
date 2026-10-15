import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

// Dos roles con cookies distintas en un único Chrome y un proyecto desechable.
export async function verifyScopeRealtime({evaluate,navigate,waitFor,output,account,call,send,base,targetId}) {
  const state = JSON.parse(await readFile('target/scope-tests/state.json','utf8'));
  const project = state.conversation_project;
  assert(project);
  const checks=[];
  const api = async (ev,path,body) => {
    const result = await ev(`fetch('/api/v1'+${JSON.stringify(path)},{credentials:'include',${body ? `method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(${JSON.stringify(body)}),`:''}}).then(async r=>({status:r.status,body:await r.json()}))`);
    assert.equal(result.status,200,JSON.stringify(result.body));
    return result.body;
  };
  assert.equal((await api(evaluate,'/users/me')).email,'scope-tests@horbyta.test');
  await navigate(`/projects/${project}?conversation=1&revision=1`);
  await waitFor("document.querySelectorAll('.scope-messages .q-message').length===100",'100 mensajes iniciales');
  assert(await evaluate("(()=>{const el=document.querySelector('.scope-messages');return el.scrollHeight>el.clientHeight && el.clientHeight>=240 && el.clientHeight<=600})()"));
  const initialFirst=await evaluate("document.querySelector('.scope-messages .q-message .scope-message-text').textContent");
  await evaluate("document.querySelector('.scope-messages').scrollTop=0");
  await waitFor(`document.querySelector('.scope-messages .q-message .scope-message-text').textContent!==${JSON.stringify(initialFirst)}`,'Ventana antigua sustituye recientes');
  const olderFirst=await evaluate("document.querySelector('.scope-messages .q-message .scope-message-text').textContent");
  assert.equal(await evaluate("document.querySelectorAll('.scope-messages .q-message').length"),100);
  checks.push('100 mensajes activos; scroll anterior sustituye ventana; altura y scroll internos');
  // Las cookies profesional/cliente tienen nombres distintos; dos tabs conservan ambos roles.
  const clientTarget=await send('Target.createTarget',{url:'about:blank'});
  const {sessionId}=await send('Target.attachToTarget',{targetId:clientTarget.targetId,flatten:true});
  const cc=(method,params={})=>send(method,params,sessionId);
  const ce=async(expression)=>{
    const r=await cc('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});
    if(r.exceptionDetails)throw Error(JSON.stringify(r.exceptionDetails));
    return r.result.value;
  };
  const cw=async(expression,label)=>{
    const deadline=Date.now()+25000;
    while(Date.now()<deadline){if(await ce(expression))return;await delay(150);}
    throw Error(`Esperando cliente: ${label}`);
  };
  try {
    await cc('Page.enable');await cc('Runtime.enable');
    await cc('Emulation.setDeviceMetricsOverride',{width:1440,height:1000,deviceScaleFactor:1,mobile:false});
    await cc('Page.addScriptToEvaluateOnNewDocument',{source:"window.__sockets=[];const Native=window.WebSocket;window.WebSocket=class extends Native{constructor(...args){super(...args);window.__sockets.push(this)}}"});
    await cc('Page.navigate',{url:new URL('/login',base).href});
    await cw("!!document.querySelector('.q-page')",'Login');
    await api(ce,'/auth/login-client',{user_id:state.owner,email:state.client_email,password:account.password});
    await cc('Page.navigate',{url:new URL(`/client/projects/${project}?conversation=1&revision=1`,base).href});
    await cw("document.querySelectorAll('.scope-messages .q-message').length===100",'Ventana cliente');
    await cw("window.__sockets.some(s=>s.readyState===1)",'Websocket cliente');
    await cc('Emulation.setFocusEmulationEnabled',{enabled:true});
    await send('Target.activateTarget',{targetId:clientTarget.targetId});
    await cw('document.hasFocus()', 'Foco cliente');
    const marker=`Enlace https://example.com/docs. Prueba ${Date.now()}`;
    await api(evaluate,`/scope/projects/${project}/feedback`,{revision:1,kind:'comment',text:marker,attachment_ids:[]});
    await cw(`document.querySelector('.scope-messages').textContent.includes(${JSON.stringify(marker)})`,'Actualización websocket');
    assert.equal(await ce("document.querySelectorAll('.scope-messages .q-message').length"),100);
    assert(await ce("!!document.querySelector('.scope-messages a[href=\"https://example.com/docs\"]')"));
    await delay(400);
    assert.equal(await ce("document.querySelectorAll('.q-notification').length"),0);
    checks.push('Websocket cliente actualiza sin recargar; link seguro; sin toast con conversación focalizada');
    assert.equal(await evaluate("document.querySelector('.scope-messages .q-message .scope-message-text').textContent"),olderFirst);
    await evaluate("Array.from(document.querySelectorAll('.scope-conversation button')).find(b=>b.textContent.includes('Ir a los últimos mensajes')).click()");
    await waitFor(`document.querySelector('.scope-messages').textContent.includes(${JSON.stringify(marker)})`,'Regreso a recientes');
    assert.equal(await evaluate("document.querySelectorAll('.scope-messages .q-message').length"),100);
    checks.push('Llegada nueva conserva historial consultado y permite regresar a últimos');
    await ce("Array.from(document.querySelectorAll('.scope-tabs .q-tab')).find(b=>b.textContent.includes('Alcance')).click()");
    await api(evaluate,`/scope/projects/${project}/feedback`,{revision:1,kind:'comment',text:'Aviso fuera de conversación',attachment_ids:[]});
    await cw("document.querySelectorAll('.q-notification').length>0",'Notificación fuera de pestaña');
    checks.push('Toast cliente únicamente fuera de conversación');
    await ce("Array.from(document.querySelectorAll('.scope-tabs .q-tab')).find(b=>b.textContent.includes('Conversación')).click()");
    await cw("document.querySelector('.scope-messages').textContent.includes('Aviso fuera de conversación')",'Entrada obtiene último slice');
    await call('Emulation.setFocusEmulationEnabled',{enabled:true});
    await send('Target.activateTarget',{targetId});
    await waitFor('document.hasFocus()', 'Foco profesional');
    await api(ce,`/scope/client/projects/${project}/feedback`,{revision:1,kind:'comment',text:'Respuesta realtime del cliente',attachment_ids:[]});
    await waitFor("document.querySelector('.scope-messages').textContent.includes('Respuesta realtime del cliente')",'Websocket profesional');
    await delay(400);
    assert.equal(await evaluate("document.querySelectorAll('.q-notification').length"),0);
    checks.push('Websocket profesional compartido actualiza y silencia aviso en conversación');
    await evaluate("Array.from(document.querySelectorAll('.scope-tabs .q-tab')).find(b=>b.textContent.includes('Alcance')).click()");
    await api(ce,`/scope/client/projects/${project}/feedback`,{revision:1,kind:'comment',text:'Aviso profesional fuera de conversación',attachment_ids:[]});
    await waitFor("document.querySelectorAll('.q-notification').length>0",'Notificación profesional fuera de pestaña');
    checks.push('Aviso profesional visible fuera de conversación');
    await cc('Emulation.setFocusEmulationEnabled',{enabled:true});
    await send('Target.activateTarget',{targetId:clientTarget.targetId});
    await ce("(()=>{const dt=new DataTransfer();dt.items.add(new File(['# Archivo de conversación\\n\\n**Markdown renderizado**'],'chat-prueba.md',{type:'text/markdown'}));const input=document.querySelector('.scope-composer input[type=file]');input.files=dt.files;input.dispatchEvent(new Event('change',{bubbles:true}));})()");
    await cw("document.querySelector('.scope-composer').textContent.includes('chat-prueba.md')&&!document.querySelector('.scope-composer .q-btn--loading')",'Adjunto subido');
    await ce("Array.from(document.querySelectorAll('.scope-composer button')).find(b=>b.textContent.trim()==='Enviar').click()");
    await cw("!!Array.from(document.querySelectorAll('.scope-message-files button')).find(b=>b.textContent.includes('chat-prueba.md'))",'Archivo enviado sin texto');
    await ce("Array.from(document.querySelectorAll('.scope-message-files button')).find(b=>b.textContent.includes('chat-prueba.md')).click()");
    await cw("document.querySelector('.attachment-preview-dialog h1')?.textContent==='Archivo de conversación'",'Markdown modal');
    const screenshot=await cc('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});
    await writeFile(join(output,'scope-chat-attachment.png'),Buffer.from(screenshot.data,'base64'));
    await cc('Input.dispatchKeyEvent',{type:'keyDown',key:'Escape',code:'Escape',windowsVirtualKeyCode:27});
    await cc('Input.dispatchKeyEvent',{type:'keyUp',key:'Escape',code:'Escape',windowsVirtualKeyCode:27});
    await cw("!document.querySelector('.attachment-preview-dialog')",'Cerrar modal');
    await ce("(()=>{const dt=new DataTransfer();dt.items.add(new File(['audio'],'prohibido.mp3',{type:'audio/mpeg'}));const input=document.querySelector('.scope-composer input[type=file]');input.files=dt.files;input.dispatchEvent(new Event('change',{bubbles:true}));})()");
    await cw("!!document.querySelector('.scope-composer [role=alert]')",'Audio bloqueado');
    assert.equal(await ce("document.querySelectorAll('.scope-composer .q-chip').length"),0);
    checks.push('Adjuntos sin texto, Markdown renderizado en modal y audio bloqueado');
    await ce("window.__sockets.filter(s=>s.readyState===1).forEach(s=>s.close())");
    await api(evaluate,`/scope/projects/${project}/feedback`,{revision:1,kind:'comment',text:'Mensaje durante reconexión',attachment_ids:[]});
    await cw("document.querySelector('.scope-messages').textContent.includes('Mensaje durante reconexión')",'Resincronización tras reconectar');
    checks.push('Reconexión recupera mensajes perdidos con ventana acotada');
    assert.equal(await ce("document.querySelectorAll('.scope-messages .q-message').length"),100);
    const chat=await cc('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});
    await writeFile(join(output,'scope-chat-client.png'),Buffer.from(chat.data,'base64'));
    await writeFile(join(output,'scope-realtime.json'),JSON.stringify({project,checks},null,2));
    console.log(`Conversación realtime: ${checks.length} comprobaciones OK`);
  } finally {await send('Target.closeTarget',{targetId:clientTarget.targetId});}
}
