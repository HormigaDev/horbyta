import { writeFile } from "node:fs/promises";
import { join } from "node:path";

// Only the dedicated local test account may create these disposable records.
export async function verifyWorkflows({
  call,
  evaluate,
  navigate,
  waitFor,
  output,
}) {
  const checks = [];
  const taskIds = [];
  let projectId, invoiceId, paymentId;
  const check = async (label, expression) => {
    if (!(await evaluate(expression)))
      throw Error(`Workflow verification failed: ${label}`);
    checks.push(label);
    console.log(`Verified: ${label}`);
  };
  const click = async (scope, text) => {
    await evaluate(
      `Array.from(document.querySelectorAll(${JSON.stringify(scope)})).find(b=>b.textContent.includes(${JSON.stringify(text)})).click()`,
    );
    if (scope.includes("[role=option]"))
      await waitFor(
        "!document.querySelector('.q-menu [role=option]')",
        "select menu closes",
      );
  };
  const type = (label, value) =>
    evaluate(
      `(() => {const field=Array.from(document.querySelectorAll('.q-field')).find(el=>el.querySelector('.q-field__label')?.textContent===${JSON.stringify(label)});const input=field.querySelector('input,textarea');input.value=${JSON.stringify(value)};input.dispatchEvent(new Event('input',{bubbles:true}));input.dispatchEvent(new Event('change',{bubbles:true}));})()`,
    );
  const key = async (key, code, windowsVirtualKeyCode) => {
    for (const type of ["keyDown", "keyUp"])
      await call("Input.dispatchKeyEvent", {
        type,
        key,
        code,
        windowsVirtualKeyCode,
      });
  };
  const api = (path, method, body) =>
    evaluate(
      `fetch('/api/v1${path}',{method:${JSON.stringify(method)},headers:{'Content-Type':'application/json'},${body ? `body:JSON.stringify(${JSON.stringify(body)}),` : ""}}).then(async r=>{if(!r.ok)throw Error('Test API status '+r.status);return r.json();})`,
    );
  await check(
    "Dedicated workflow account",
    "fetch('/api/v1/users/me').then(r=>r.json()).then(u=>u.email==='ui-tests@horbyta.test')",
  );
  const originalTable = await evaluate(
    "fetch('/api/v1/users/me/ui-state/datatable/tasks_table').then(r=>r.ok?r.json():null)",
  );
  const originalPreferences = await evaluate(
    "fetch('/api/v1/users/me').then(r=>r.json()).then(u=>u.preferences)",
  );
  let changedRegional = false;
  try {
    await navigate("/tasks");
    await evaluate(
      "document.querySelector('button[aria-label=\"Configurar columnas\"]').focus()",
    );
    for (const type of ["keyDown", "keyUp"])
      await call("Input.dispatchKeyEvent", {
        type,
        key: "Enter",
        code: "Enter",
        windowsVirtualKeyCode: 13,
      });
    await waitFor(
      "!!document.querySelector('.q-dialog')",
      "keyboard column dialog",
    );
    for (const type of ["keyDown", "keyUp"])
      await call("Input.dispatchKeyEvent", {
        type,
        key: "Escape",
        code: "Escape",
        windowsVirtualKeyCode: 27,
      });
    await waitFor("!document.querySelector('.q-dialog')", "dialog Escape");
    await check(
      "Dialog restores focus to its trigger",
      "document.activeElement.getAttribute('aria-label')==='Configurar columnas'",
    );
    await evaluate(
      "document.querySelector('button[aria-label=\"Menú de cuenta\"]').focus()",
    );
    await key("Enter", "Enter", 13);
    await waitFor("!!document.querySelector('.q-menu')", "account menu");
    await key("Escape", "Escape", 27);
    await waitFor("!document.querySelector('.q-menu')", "account Escape");
    await check(
      "Account menu restores keyboard focus",
      "document.activeElement.getAttribute('aria-label')==='Menú de cuenta'",
    );
    await evaluate(
      "document.querySelector('button[aria-label=Notificaciones]').focus()",
    );
    await key("Enter", "Enter", 13);
    await waitFor(
      "!!document.querySelector('.q-dialog')",
      "notifications dialog",
    );
    await evaluate(
      "document.querySelector('.q-dialog button[aria-label=Cerrar]').focus()",
    );
    await key("Enter", "Enter", 13);
    await waitFor(
      "!document.querySelector('.q-dialog')",
      "notifications close",
    );
    await check(
      "Notifications close restores keyboard focus",
      "document.activeElement.getAttribute('aria-label')==='Notificaciones'",
    );
    projectId = (
      await api("/projects", "POST", {
        name: "[UI] Temporary workflow BRL",
        billable: true,
        project_billing: {
          currency_type: "BRL",
          billing_type: "fixed",
          fixed_price: "1234.50",
        },
      })
    ).id;
    for (let index = 0; index < 6; index++)
      taskIds.push(
        (
          await api("/tasks", "POST", {
            title: `[UI] Workflow ${index}`,
            project_id: projectId,
            status: "todo",
            priority: 1,
            complexity: 1,
            block_parent: false,
          })
        ).id,
      );
    await evaluate(
      "(()=>{const input=document.querySelector('input[aria-label=\"Buscar tareas\"]');input.focus();input.value='[UI] Workflow';input.dispatchEvent(new Event('input',{bubbles:true}));})()",
    );
    await waitFor(
      "Array.from(document.querySelectorAll('.q-menu .q-item__label')).some(el=>el.textContent.includes('[UI] Workflow'))",
      "task search",
    );
    await key("ArrowDown", "ArrowDown", 40);
    await key("Enter", "Enter", 13);
    await waitFor(
      "location.pathname.startsWith('/tasks/') && !!document.querySelector('.detail-page')",
      "keyboard search selection",
    );
    checks.push("Search finds matching tasks and opens by keyboard");
    await navigate("/tasks");
    await evaluate(
      "document.querySelector('.q-table__bottom .q-select').click()",
    );
    await waitFor(
      "!!document.querySelector('.q-menu [role=option]')",
      "rows per page menu",
    );
    await click(".q-menu [role=option]", "5");
    await waitFor(
      "document.querySelectorAll('.q-table tbody tr').length===5",
      "five rows",
    );
    await evaluate(
      "Array.from(document.querySelectorAll('.q-table__bottom button')).find(b=>b.querySelector('.q-icon')?.textContent==='chevron_right').click()",
    );
    await waitFor(
      "!document.querySelector('.q-linear-progress') && document.querySelectorAll('.q-table tbody tr').length>0 && document.querySelectorAll('.q-table tbody tr').length<5",
      "second page",
    );
    checks.push("Table paginates real records");
    await evaluate(
      "Array.from(document.querySelectorAll('.q-table thead th')).find(el=>el.textContent.includes('Título')).focus()",
    );
    await key("Enter", "Enter", 13);
    await waitFor(
      "document.querySelector('.q-table thead th.sorted')?.textContent.includes('Título')",
      "title sorting",
    );
    checks.push("Table sorts by title with keyboard");
    await evaluate(
      "document.querySelector('button[aria-label=Filtros]').click()",
    );
    await click(".data-table-wrapper button", "Agregar filtro");
    await evaluate(
      "Array.from(document.querySelectorAll('.q-field')).find(el=>el.querySelector('.q-field__label')?.textContent==='Columna').click()",
    );
    await waitFor(
      "!!document.querySelector('.q-menu [role=option]')",
      "filter column menu",
    );
    await click(".q-menu [role=option]", "Título");
    await evaluate(
      "Array.from(document.querySelectorAll('.q-field')).find(el=>el.querySelector('.q-field__label')?.textContent==='Operador').click()",
    );
    await waitFor(
      "!!document.querySelector('.q-menu [role=option]')",
      "filter operator menu",
    );
    await click(".q-menu [role=option]", "Igual");
    await type("Valor", "[UI] Workflow 0");
    await check(
      "Unapplied filter stays a draft",
      "!document.querySelector('.applied-filter') && Array.from(document.querySelectorAll('[role=status]')).some(el=>el.textContent.includes('sin aplicar'))",
    );
    await click(".data-table-wrapper button", "Aplicar filtros");
    await waitFor(
      "document.querySelectorAll('.q-table tbody tr').length===1 && !!document.querySelector('.applied-filter')",
      "filter application",
    );
    await navigate("/tasks");
    await check(
      "Applied filters survive reload",
      "!!document.querySelector('.applied-filter') && document.querySelectorAll('.q-table tbody tr').length===1",
    );
    await click(".data-table-wrapper button", "Limpiar todos");
    await waitFor(
      "!document.querySelector('.applied-filter') && document.querySelectorAll('.q-table tbody tr').length>1",
      "filter reset",
    );
    checks.push("Table clears filters");
    await evaluate(
      "document.querySelector('button[aria-label=\"Configurar columnas\"]').click()",
    );
    await waitFor(
      "!!document.querySelector('.q-dialog')",
      "column preferences",
    );
    const oldPriority = await evaluate(
      "Array.from(document.querySelectorAll('.q-dialog .q-item')).find(el=>el.textContent.includes('Prioridad')).querySelector('[role=checkbox]').getAttribute('aria-checked')",
    );
    await evaluate(
      "Array.from(document.querySelectorAll('.q-dialog .q-item')).find(el=>el.textContent.includes('Prioridad')).click()",
    );
    await click(".q-dialog button", "Guardar");
    await waitFor("!document.querySelector('.q-dialog')", "column save");
    await navigate("/tasks");
    await check(
      "Column choice survives reload",
      `Array.from(document.querySelectorAll('.q-table thead th')).some(el=>el.textContent.includes('Prioridad')) === ${oldPriority !== "true"}`,
    );
    await navigate("/invoices/new");
    await evaluate("document.querySelector('.form-page .q-select').click()");
    await waitFor(
      "Array.from(document.querySelectorAll('.q-menu [role=option]')).some(el=>el.textContent.includes('Temporary workflow BRL'))",
      "invoice project",
    );
    await click(".q-menu [role=option]", "Temporary workflow BRL");
    await type("Monto adicional", "1234.50");
    await type("Fecha de vencimiento", "21/10/2026");
    await click(".form-actions button", "Guardar");
    await waitFor(
      "location.pathname.startsWith('/invoices/') && !location.pathname.endsWith('/new') && !!document.querySelector('.detail-page')",
      "invoice creation",
    );
    invoiceId = await evaluate('location.pathname.split("/").pop()');
    await check(
      "Invoice keeps amount and project currency",
      `fetch('/api/v1/invoices/${invoiceId}').then(r=>r.json()).then(v=>Number(v.amount)===1234.5 && v.currency_type==='BRL' && v.status==='draft')`,
    );
    await api("/users/me/preferences", "PATCH", {
      number_format: {
        thousand_separator: ".",
        decimal_separator: ",",
        decimal_quantity: 2,
      },
      date_format: "MM/DD/YYYY",
    });
    changedRegional = true;
    await navigate(`/invoices/${invoiceId}/edit`);
    await check(
      "Invoice date follows regional preference",
      "Array.from(document.querySelectorAll('.q-field')).find(el=>el.querySelector('.q-field__label')?.textContent==='Fecha de vencimiento').querySelector('input').value==='10/21/2026'",
    );
    await type("Monto adicional", "1250,75");
    await type("Fecha de vencimiento", "10/22/2026");
    await click(".form-actions button", "Guardar");
    await waitFor("!!document.querySelector('.detail-page')", "invoice update");
    await check(
      "Draft invoice preserves regional decimal and date",
      `fetch('/api/v1/invoices/${invoiceId}').then(r=>r.json()).then(v=>Number(v.amount)===1250.75 && v.due_date==='2026-10-22')`,
    );
    await navigate("/payments/new");
    await evaluate("document.querySelector('.form-page .q-select').click()");
    await waitFor(
      "Array.from(document.querySelectorAll('.q-menu [role=option]')).some(el=>el.textContent.includes('Temporary workflow BRL'))",
      "payment project",
    );
    await click(".q-menu [role=option]", "Temporary workflow BRL");
    await type("Monto *", "50,25");
    await click(".form-actions button", "Guardar");
    await waitFor(
      "location.pathname.startsWith('/payments/') && !location.pathname.endsWith('/new') && !!document.querySelector('.detail-page')",
      "payment creation",
    );
    paymentId = await evaluate('location.pathname.split("/").pop()');
    await check(
      "Payment keeps amount and currency",
      `fetch('/api/v1/payments/${paymentId}').then(r=>r.json()).then(v=>Number(v.amount)===50.25 && v.currency_type==='BRL')`,
    );
    await navigate(`/payments/${paymentId}/edit`);
    await type("Observaciones", "[UI] Edited payment");
    await click(".form-actions button", "Guardar");
    await waitFor("!!document.querySelector('.detail-page')", "payment update");
    await check(
      "Payment edits permitted metadata",
      `fetch('/api/v1/payments/${paymentId}').then(r=>r.json()).then(v=>v.notes==='[UI] Edited payment' && Number(v.amount)===50.25)`,
    );
    await navigate(`/tasks/${taskIds[0]}/edit`);
    await evaluate("document.querySelector('.form-page input').focus()");
    for (let index = 0; index < 32; index++) {
      await key("Tab", "Tab", 9);
      const obscured = await evaluate(
        "new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>{const a=document.activeElement;const p=document.querySelector('.form-page');const footer=document.querySelector('.form-actions');const rect=a.getBoundingClientRect();resolve(p.contains(a)&&!footer.contains(a)&&rect.height>0&&rect.bottom>footer.getBoundingClientRect().top+1);})))",
      );
      if (obscured)
        throw Error("Sticky form actions obscure the focused control");
    }
    checks.push("Long form keyboard traversal keeps focus above actions");
    await evaluate(
      "(()=>{const dt=new DataTransfer();dt.items.add(new File(['UI attachment verification'], 'ui-verification-with-a-long-readable-name.txt',{type:'text/plain'}));const input=document.querySelector('.markdown-file-input');input.files=dt.files;input.dispatchEvent(new Event('change',{bubbles:true}));})()",
    );
    await waitFor(
      "document.querySelector('.attachment-name')?.textContent.includes('ui-verification')",
      "pending file",
    );
    await click(".form-actions button", "Guardar");
    await waitFor(
      "!!document.querySelector('.detail-page')",
      "attachment save",
    );
    const attachment = await evaluate(
      `import('/src/services/attachments.ts').then(m=>m.AttachmentsService.searchByTask('${taskIds[0]}',{page:1,limit:10,order_by:'created_at',order_asc:false,filters:[]})).then(r=>r.items[0])`,
    );
    if (!attachment) throw Error("Attachment was not uploaded");
    await check(
      "Saved attachment can be downloaded",
      `fetch('/api/v1/attachments/${attachment.id}/raw').then(r=>r.text()).then(t=>t==='UI attachment verification')`,
    );
    await api(`/attachments/${attachment.id}`, "DELETE");
    checks.push("Attachment uploads and deletes with the existing API");
  } catch (error) {
    const { data } = await call("Page.captureScreenshot", {
      format: "png",
      captureBeyondViewport: false,
    });
    await writeFile(
      join(output, "workflow-failure.png"),
      Buffer.from(data, "base64"),
    );
    await writeFile(
      join(output, "workflow-failure.json"),
      JSON.stringify(
        await evaluate(
          "({path:location.pathname,alerts:Array.from(document.querySelectorAll('.q-notification,.q-dialog')).map(el=>el.textContent),focus:document.activeElement.tagName})",
        ),
        null,
        2,
      ),
    );
    throw error;
  } finally {
    if (changedRegional)
      await api("/users/me/preferences", "PATCH", {
        number_format: originalPreferences.number_format,
        date_format: originalPreferences.date_format,
      });
    if (paymentId) await api(`/payments/${paymentId}`, "DELETE");
    if (invoiceId) await api(`/invoices/${invoiceId}`, "DELETE");
    for (const id of taskIds) await api(`/tasks/${id}`, "DELETE");
    if (projectId) await api(`/projects/${projectId}`, "DELETE");
    const value = originalTable?.value;
    if (value)
      await api("/users/me/ui-state/datatable/tasks_table", "PUT", { value });
    else await api("/users/me/ui-state/datatable/tasks_table", "DELETE");
  }
  await writeFile(
    join(output, "workflow-verification.json"),
    JSON.stringify({ checkedAt: new Date().toISOString(), checks }, null, 2),
  );
}
