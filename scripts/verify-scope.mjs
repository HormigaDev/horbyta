import { readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

// Opera exclusivamente sobre las identidades generadas por el laboratorio aislado.
export async function verifyScope({
  call,
  evaluate,
  navigate,
  waitFor,
  output,
  account,
}) {
  const state = JSON.parse(
    await readFile("target/scope-tests/state.json", "utf8"),
  );
  const checks = [];
  const api = async (path, method = "GET", body, expected = 200) => {
    const result = await evaluate(
      `fetch('/api/v1'+${JSON.stringify(path)},{method:${JSON.stringify(method)},credentials:'include',headers:{'Content-Type':'application/json'},${body === undefined ? "" : `body:JSON.stringify(${JSON.stringify(body)}),`}}).then(async r=>({status:r.status,body:await r.json()}))`,
    );
    if (result.status !== expected)
      throw Error(
        `${method} ${path}: ${result.status}, esperado ${expected}: ${JSON.stringify(result.body)}`,
      );
    return result.body;
  };
  const assert = (ok, label) => {
    if (!ok) throw Error(label);
    checks.push(label);
  };
  const click = async (label, selector = "button, .q-item") => {
    const expression = `Array.from(document.querySelectorAll(${JSON.stringify(selector)})).find(e=>e.checkVisibility()&&e.textContent.trim().endsWith(${JSON.stringify(label)}))`;
    await waitFor(`!!(${expression})`, label);
    await evaluate(`(${expression}).click()`);
  };
  const screenshot = async (label) => {
    // Espera las transiciones de Quasar antes de capturar menús y expansiones.
    await evaluate('new Promise(resolve => setTimeout(resolve, 550))');
    const shot = await call("Page.captureScreenshot", {
      format: "png",
      captureBeyondViewport: true,
    });
    await writeFile(
      join(output, `scope-${label}.png`),
      Buffer.from(shot.data, "base64"),
    );
  };
  const owner = await api("/users/me");
  assert(
    owner.id === state.owner && owner.email === "scope-tests@horbyta.test",
    "Identidad exclusiva del esquema de laboratorio",
  );
  // Cierra propuestas de un intento anterior únicamente dentro de esta cuenta aislada.
  await api('/auth/login-client','POST',{user_id:state.owner,email:state.client_email,password:account.password});
  for(const notice of await api('/scope/client/pending')) {
    if(notice.kind==='baseline') {
      const old=await api(`/scope/client/projects/${notice.record_id}`);
      const offer=old.revisions.find(r=>r.revision===notice.revision);
      if(offer?.status==='pending')await api(`/scope/client/projects/${notice.record_id}/feedback`,'POST',{revision:notice.revision,kind:'reject',text:'Cierre de un intento anterior del laboratorio aislado',document_id:null});
    } else if (notice.kind === 'change') {
      const old = await api(`/scope/client/changes/${notice.record_id}`);
      if (old.current.status === 'assessed') {
        await api(`/change-requests/client/${notice.record_id}/decline`, 'PATCH', {
          version: old.current.version,
          decline_reason: 'not_necessary_anymore',
        });
      }
    }
  }
  const project = await api("/projects", "POST", {
    name: "[Alcance] Recorrido real del portal",
    description: "Proyecto aislado de prueba",
    client_id: state.client,
    billable: false,
  });
  const base = await api("/tasks", "POST", {
    project_id: project.id,
    title: "Entrega incluida",
    status: "todo",
    priority: 1,
    complexity: 1,
    block_parent: true,
  });
  await navigate(`/projects/${project.id}`);
  await click("Preparar alcance");
  const sections = [
    "Tres páginas informativas",
    "Navegación y contraste verificables",
    "Una revisión de contenido",
    "Sin tienda ni pagos",
    "Acceso autenticado y diseño adaptable",
  ];
  await evaluate(
    `(()=>{const inputs=Array.from(document.querySelectorAll('.q-page form textarea'));const values=${JSON.stringify(sections)};if(inputs.length!==5)throw Error('Faltan secciones del alcance');inputs.forEach((input,i)=>{input.value=values[i];input.dispatchEvent(new Event('input',{bubbles:true}));});})()`,
  );
  await click("Publicar para el cliente");
  await waitFor(
    "!Array.from(document.querySelectorAll('button')).some(b=>b.textContent.includes('Publicar para el cliente'))",
    "publicación persistida",
  );
  assert(
    (await api(`/scope/projects/${project.id}`)).last_revision === 1,
    "Publicación del alcance desde el formulario real",
  );
  await screenshot("alcance-publicado");
  await api("/auth/login-client", "POST", {
    user_id: state.owner,
    email: state.client_email,
    password: account.password,
  });
  await navigate(`/client/projects/${project.id}`);
  await click("Aceptar este alcance");
  await click("Aceptar este alcance", ".q-dialog button");
  await waitFor(
    "document.querySelector('.q-page').textContent.includes('Acuerdo activo')",
    "aceptación del alcance",
  );
  assert(
    (await api(`/scope/client/projects/${project.id}`)).active,
    "Aceptación autenticada y revisión conservada",
  );
  await screenshot("alcance-aceptado");
  const cr = await api("/change-requests", "POST", {
    project_id: project.id,
    client_id: state.client,
    title: "Extra de prueba rechazable",
    client_request_description: "Añadir sección excluida",
  });
  const submitted = await api(`/change-requests/${cr.id}/submit`, "PATCH", {
    version: cr.version,
  });
  await navigate(`/change-requests/${cr.id}`);
  await click("Evaluar");
  await waitFor("!!document.querySelector('.q-dialog textarea')", "evaluación");
  await evaluate(
    "(()=>{const input=document.querySelector('.q-dialog textarea');input.value='Extra sin costo ni plazo adicional';input.dispatchEvent(new Event('input',{bubbles:true}));})()",
  );
  await click("Guardar evaluación", ".q-dialog button");
  await waitFor("!document.querySelector('.q-dialog')", "evaluación publicada");
  const offer = await api(`/scope/changes/${cr.id}`);
  assert(
    offer.current.status === "assessed" &&
      offer.current.version > submitted.version,
    "Evaluación real con versión y términos publicados",
  );
  const extra = await api("/tasks", "POST", {
    project_id: project.id,
    title: "Extra aún bloqueado",
    status: "todo",
    priority: 1,
    complexity: 1,
    block_parent: true,
  });
  await navigate(`/tasks/${extra.id}`);
  await evaluate(
    "Array.from(document.querySelectorAll('.q-field')).find(e=>e.textContent.includes('Alcance o solicitud')).click()",
  );
  await waitFor(
    "Array.from(document.querySelectorAll('.q-menu .q-item')).some(e=>e.textContent.includes('Extra de prueba rechazable'))",
    "selector de origen",
  );
  await evaluate(
    "Array.from(document.querySelectorAll('.q-menu .q-item')).find(e=>e.textContent.includes('Extra de prueba rechazable')).click()",
  );
  await evaluate(
    "(()=>{const input=document.querySelector('.q-page form textarea');input.value='Sección excluida del acuerdo';input.dispatchEvent(new Event('input',{bubbles:true}));})()",
  );
  await click("Clasificar trabajo");
  await waitFor(
    "!Array.from(document.querySelectorAll('button')).some(e=>e.textContent.includes('Clasificar trabajo'))",
    "clasificación persistida",
  );
  assert(
    !(await api(`/scope/tasks/${extra.id}`)).allowed,
    "Clasificación desde UI conserva el extra bloqueado",
  );
  await api(`/tasks/${extra.id}`, "PATCH", { status: "in_progress" }, 409);
  await screenshot("tarea-bloqueada");
  await navigate(`/client/change-requests/${cr.id}`);
  await click("Declinar");
  await evaluate("document.querySelector('.q-dialog .q-select').click()");
  await waitFor(
    "!!document.querySelector('.q-menu .q-item')",
    "motivos de rechazo",
  );
  await evaluate(
    "Array.from(document.querySelectorAll('.q-menu .q-item')).find(e=>/necesari/i.test(e.textContent)).click()",
  );
  await click("Declinar", ".q-dialog button");
  await waitFor("!document.querySelector('.q-dialog')", "rechazo persistido");
  assert(
    (await api(`/scope/client/changes/${cr.id}`)).current.status === "declined",
    "Rechazo conserva el alcance y bloquea el extra",
  );
  assert(
    !(await api(`/scope/tasks/${extra.id}`)).allowed,
    "El extra rechazado sigue bloqueado",
  );
  await api(`/tasks/${base.id}`, "PATCH", { status: "in_progress" });
  checks.push("El trabajo original sigue ejecutable después del rechazo");
  // Resuelve solo el pendiente heredado del laboratorio; las ofertas del recorrido son nuevas.
  const previous = await api(`/scope/client/changes/${state.pending_change}`);
  if (previous.current.status === "assessed")
    await api(
      `/change-requests/client/${state.pending_change}/decline`,
      "PATCH",
      {
        version: previous.current.version,
        decline_reason: "not_necessary_anymore",
      },
    );
  const positive = await api("/change-requests", "POST", {
    project_id: state.project,
    client_id: state.client,
    title: "Suplemento de recorrido browserless",
    client_request_description: "Extra con precio y plazo definidos",
  });
  const submittedPositive = await api(
    `/change-requests/${positive.id}/submit`,
    "PATCH",
    { version: positive.version },
  );
  await api(`/change-requests/${positive.id}/assess`, "PATCH", {
    version: submittedPositive.version,
    technical_assessment: "Oferta de prueba",
    impact_hours: "1",
    impact_cost: "10",
    schedule_impact_days: 2,
    target_milestone_id: state.milestone,
  });
  const positiveTask = await api("/tasks", "POST", {
    project_id: state.project,
    title: "Trabajo del suplemento",
    status: "todo",
    priority: 1,
    complexity: 1,
    block_parent: true,
  });
  await api(`/scope/tasks/${positiveTask.id}`, "POST", {
    change_request_id: positive.id,
    reason: "Extra del recorrido aislado",
  });
  const positiveId = positive.id;
  // La segunda oferta se modifica mientras el cliente conserva su versión vista.
  await navigate(`/client/change-requests/${positiveId}`);
  await waitFor(
    "Array.from(document.querySelectorAll('.q-item')).some(e=>e.textContent.trim().endsWith('Aprobar'))",
    "oferta cargada",
  );
  await evaluate(
    "(()=>{const input=document.querySelector('.q-page form textarea');input.value='Revisar el presupuesto antes de aceptar';input.dispatchEvent(new Event('input',{bubbles:true}));})()",
  );
  await click("Agregar aclaración");
  await waitFor(
    "document.querySelector('.q-page form textarea').value===''",
    "aclaración conservada",
  );
  const negotiation = await api(`/scope/client/changes/${positiveId}`);
  assert(
    negotiation.events.some(
      (e) =>
        e.kind === "comment" &&
        e.actor_kind === "client" &&
        e.body.text === "Revisar el presupuesto antes de aceptar",
    ),
    "Aclaración conservada con actor y revisión",
  );
  const old = await api(`/scope/changes/${positiveId}`);
  await api(`/change-requests/${positiveId}/assess`, "PATCH", {
    version: old.current.version,
    technical_assessment: "Oferta revisada durante la lectura",
    impact_hours: "2",
    impact_cost: "20.1250",
    schedule_impact_days: 2,
    target_milestone_id: state.milestone,
  });
  await click("Aprobar");
  await waitFor(
    "document.querySelector('.q-page').textContent.includes('20.1250')",
    "recarga de la oferta tras conflicto",
  );
  assert(
    !(await api(`/scope/client/changes/${positiveId}`)).effect,
    "Pantalla antigua genera conflicto y ningún efecto financiero",
  );
  await screenshot("oferta-revisada");
  await click("Aprobar");
  await waitFor(
    "document.querySelector('.q-page').textContent.includes('Borrador pendiente de emisión')",
    "factura derivada",
  );
  const accepted = await api(`/scope/client/changes/${positiveId}`);
  assert(
    accepted.effect &&
      accepted.effect.amount === "20.1250" &&
      accepted.effect.currency_type === "EUR",
    "La aprobación aplica un único suplemento con moneda e importe exactos",
  );
  assert(
    accepted.current.schedule_after && accepted.tasks.every((t) => t.allowed),
    "La aprobación ajusta el hito y habilita las tareas",
  );
  await api(`/change-requests/client/${positiveId}/approve`, "PATCH", {
    version: old.current.version + 1,
  });
  assert(
    (await api(`/scope/client/changes/${positiveId}`)).effect.invoice_id ===
      accepted.effect.invoice_id,
    "Reintentar la aprobación conserva la misma factura",
  );
  await screenshot("aprobacion-aplicada");
  await navigate("/client");
  await waitFor(
    "document.querySelector('.q-page').textContent.includes('No tienes decisiones pendientes')",
    "bandeja persistente resuelta",
  );
  checks.push("Bandeja de pendientes se resuelve después de decidir");
  await navigate(`/client/projects/${project.id}`);
  await evaluate("Array.from(document.querySelectorAll('.scope-document button')).find(b=>b.textContent.includes('Actualizar')).focus()");
  await call('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Tab', code: 'Tab', windowsVirtualKeyCode: 9 });
  await call('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Tab', code: 'Tab', windowsVirtualKeyCode: 9 });
  assert(await evaluate("document.activeElement!==document.body && document.activeElement.checkVisibility()"), 'Navegación del alcance por teclado');
  for (const mode of ['dark', 'light']) {
    await call('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-color-scheme', value: mode }] });
    await waitFor(`document.documentElement.dataset.theme===${JSON.stringify(mode)}`, `tema ${mode}`);
    assert(await evaluate("document.querySelector('.scope-document').textContent.includes('Tres páginas informativas')"), `El acuerdo conserva su contenido en tema ${mode}`);
    await screenshot(`alcance-${mode}`);
  }
  await call('Network.emulateNetworkConditions', { offline: true, latency: 0, downloadThroughput: -1, uploadThroughput: -1 });
  try {
    await click('Actualizar acuerdos', '.scope-document button');
    await waitFor("!!document.querySelector('.scope-document [role=alert]')", 'error de conexión explícito');
    assert(await evaluate("document.querySelector('.scope-document').textContent.includes('Tres páginas informativas')"), 'La desconexión conserva el acuerdo leído y muestra el error');
  } finally {
    await call('Network.emulateNetworkConditions', { offline: false, latency: 0, downloadThroughput: -1, uploadThroughput: -1 });
  }
  await click('Volver a cargar', '.scope-document button');
  await waitFor("!document.querySelector('.scope-document [role=alert]')", 'recuperación de conexión');
  checks.push('Reintento recupera la lectura después de desconexión');
  await call("Emulation.setDeviceMetricsOverride", {
    width: 390,
    height: 844,
    deviceScaleFactor: 1,
    mobile: true,
  });
  await waitFor("document.querySelector('.scope-document').getBoundingClientRect().width > 300", 'layout móvil estabilizado');
  assert(
    await evaluate("document.documentElement.scrollWidth<=innerWidth+2 && document.querySelector('.scope-document').getBoundingClientRect().width > 300"),
    "El alcance conserva lectura móvil sin desbordamiento horizontal",
  );
  await screenshot("alcance-movil");
  await call("Emulation.setDeviceMetricsOverride", {
    width: 1440,
    height: 1000,
    deviceScaleFactor: 1,
    mobile: false,
  });
  await waitFor("document.querySelector('.scope-document').getBoundingClientRect().width > 600", 'layout de escritorio estabilizado');
  const {verifyScopeDocuments}=await import('./verify-scope-documents.mjs');
  await verifyScopeDocuments({api,call,evaluate,navigate,waitFor,click,assert,screenshot,state,output});
  await api("/auth/logout-client", "POST");
  await writeFile(
    join(output, "scope-verification.json"),
    JSON.stringify({ checkedAt: new Date().toISOString(), checks }, null, 2),
  );
  for (const label of checks) console.log(`Verificado: ${label}`);
}
