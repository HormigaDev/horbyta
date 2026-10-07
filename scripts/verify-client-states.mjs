import { writeFile } from "node:fs/promises";
import { join } from "node:path";

// Additional presentation/leave-guard coverage that works without PostgreSQL.
// These fixtures do not authenticate against a server or verify server permissions.
const owner = "00000000-0000-4000-8000-000000000001";
const projectId = "00000000-0000-4000-8000-000000000002";
const requestId = "00000000-0000-4000-8000-000000000003";
const milestoneId = "00000000-0000-4000-8000-000000000004";
const date = "2026-10-06T12:00:00Z";
const project = {
  id: projectId,
  name: "Proyecto de revisión del portal con un título largo",
  description: "## Próxima entrega\n\nComprobar **legibilidad**.",
  status: "open",
  billable: false,
  project_billing: null,
  created_at: date,
  updated_at: date,
};
const request = {
  id: requestId,
  project_id: projectId,
  project_name: project.name,
  record_number: 1,
  task_id: null,
  title: "Solicitud de revisión de accesibilidad",
  client_request_description:
    "## Revisión solicitada\n\nComprobar **contraste y foco**.",
  technical_assessment: null,
  impact_hours: "0",
  impact_cost: "0",
  schedule_impact_days: 0,
  origin: "client_portal",
  status: "submitted",
  client_approved_at: null,
  client_signature_ip: null,
  created_at: date,
  updated_at: date,
  version: 1,
};
const milestone = {
  id: milestoneId,
  name: "Revisar primera entrega",
  description: "Comprobar **objetivos**.",
  status: "pending",
  due_date: "2026-11-01T12:00:00Z",
  created_at: date,
  updated_at: date,
};
export async function installClientFixtures(call) {
  await call("Page.addScriptToEvaluateOnNewDocument", {
    source: `(()=>{
    const real=window.fetch.bind(window);window.__requests=[];window.__record=${JSON.stringify(request)};
    window.fetch=async(input,opts)=>{
      const u=new URL(typeof input==='string'?input:input.url,location.origin),p=u.pathname;
      if(!p.startsWith('/api/v1/'))return real(input,opts);
      window.__requests.push(p);
      if(p==='/api/v1/health')return Response.json({error:'no session'},{status:401});
      if(p==='/api/v1/auth/registration-status')return Response.json({allow_registration:false});
      if(p==='/api/v1/auth/client-me')return Response.json({id:'client-fixture',client_id:'client-fixture',user_id:${JSON.stringify(owner)},name:'Cliente de revisión'});
      if(p.startsWith('/api/v1/projects/client-search'))return Response.json({items:[${JSON.stringify(project)}],total:1,payload:JSON.parse(u.searchParams.get('q'))});
      if(p==='/api/v1/projects/${projectId}/client')return Response.json(${JSON.stringify(project)});
      if(p.startsWith('/api/v1/milestones/client/search-by-project/'))return Response.json({items:[${JSON.stringify(milestone)}],total:1,payload:JSON.parse(u.searchParams.get('q'))});
      if(p==='/api/v1/change-requests/client/${requestId}')return Response.json(window.__record);
      if(p.startsWith('/api/v1/change-requests/client/search'))return Response.json({items:[window.__record],total:1,payload:JSON.parse(u.searchParams.get('q'))});
      return Response.json({error:'Unconfigured browser fixture'},{status:501});
    };
  })()`,
  });
}
export async function verifyClientStates({
  call,
  evaluate,
  navigate,
  waitFor,
  output,
}) {
  const checks = [];
  const check = async (label, expression) => {
    if (!(await evaluate(expression)))
      throw Error("Client UI verification failed: " + label);
    checks.push(label);
    console.log("Verified UI fixture: " + label);
  };
  const shot = async (name) => {
    const { data } = await call("Page.captureScreenshot", {
      format: "png",
      captureBeyondViewport: false,
    });
    await writeFile(join(output, name + ".png"), Buffer.from(data, "base64"));
  };
  const click = (selector, text) =>
    evaluate(
      `Array.from(document.querySelectorAll(${JSON.stringify(selector)})).find(b=>b.textContent.includes(${JSON.stringify(text)})).click()`,
    );
  for (const width of [1440, 768, 360, 320]) {
    await call("Emulation.setDeviceMetricsOverride", {
      width,
      height: 1000,
      deviceScaleFactor: 1,
      mobile: width < 600,
    });
    for (const locale of ["es", "en", "pt"]) {
      await navigate("/client/projects/" + projectId);
      await evaluate(
        `import('/src/boot/i18n.ts').then(m=>m.setAppLocale(${JSON.stringify(locale)}))`,
      );
      await evaluate(
        "import('/src/stores/theme.ts').then(m=>m.useThemeStore().commit({...m.useThemeStore().appearance,theme_id:'forest',mode:'dark'}))",
      );
      await check(
        "Project detail " + locale + " " + width,
        "!!document.querySelector('.detail-page h1') && document.querySelectorAll('.detail-page .q-card__section > h2').length===1 && document.documentElement.scrollWidth<=innerWidth",
      );
      await check(
        "Readable dark portal section " + locale + " " + width,
        "getComputedStyle(document.querySelector('.detail-page .q-card__section > h2')).color===getComputedStyle(document.documentElement).getPropertyValue('--text-accent').trim() || (()=>{const e=document.createElement('span');e.style.color='var(--text-accent)';document.body.append(e);const color=getComputedStyle(e).color;e.remove();return color===getComputedStyle(document.querySelector('.detail-page .q-card__section > h2')).color;})()",
      );
      await shot("project-fixture-" + locale + "-" + width);
    }
  }
  await evaluate("import('/src/boot/i18n.ts').then(m=>m.setAppLocale('es'))");
  await navigate(
    "/client/milestones/" + milestoneId + "?project_id=" + projectId,
  );
  await check(
    "Milestone has semantic heading and linked project",
    "document.querySelector('.detail-page h1').textContent.includes('Revisar primera entrega') && document.documentElement.scrollWidth<=innerWidth",
  );
  await shot("milestone-fixture-mobile");
  await navigate("/client/change-requests/" + requestId);
  await check(
    "Submitted request detail shows Markdown and actions",
    "document.querySelector('.detail-page h1').textContent.includes('Solicitud de revisión') && document.querySelectorAll('.detail-page .q-card__section > h2').length===3",
  );
  await shot("request-fixture-mobile");
  await navigate("/client/change-requests/" + requestId + "/edit");
  await check(
    "Submitted request remains editable",
    "!!document.querySelector('.markdown-editor textarea') && !!document.querySelector('.form-actions button.bg-primary')",
  );
  await evaluate(
    "(()=>{const input=Array.from(document.querySelectorAll('.form-page input')).find(i=>!i.closest('.q-select'));input.value='Borrador de revisión';input.dispatchEvent(new Event('input',{bubbles:true}));})()",
  );
  await click(".form-actions button", "Cancelar");
  await waitFor("!!document.querySelector('.q-dialog')", "client leave guard");
  await click(".q-dialog button", "Quedarme");
  await waitFor("!document.querySelector('.q-dialog')", "stay on dirty form");
  await check(
    "Client leave guard keeps unsaved input",
    "location.pathname.endsWith('/edit') && Array.from(document.querySelectorAll('.form-page input')).some(i=>i.value==='Borrador de revisión')",
  );
  await click(".form-actions button", "Cancelar");
  await waitFor("!!document.querySelector('.q-dialog')", "leave confirmation");
  await click(".q-dialog button", "Salir");
  await waitFor("!location.pathname.endsWith('/edit')", "leave dirty form");
  await navigate("/client/change-requests/" + requestId + "/edit");
  await waitFor(
    "!!document.querySelector('.q-dialog')",
    "restore client draft",
  );
  await click(".q-dialog button", "Restaurar");
  await waitFor("!document.querySelector('.q-dialog')", "restore completed");
  await check(
    "Client draft restores previous input",
    "Array.from(document.querySelectorAll('.form-page input')).some(i=>i.value==='Borrador de revisión')",
  );
  await evaluate(
    "Object.keys(localStorage).filter(k=>k.startsWith('form-draft:client:')).forEach(k=>localStorage.removeItem(k))",
  );
  // Insert a cancelled draft response before component mount on this SPA route.
  await evaluate(
    "window.__record.status='cancelled';document.querySelector('.form-actions button').click()",
  );
  await waitFor(
    "!!document.querySelector('.q-dialog')",
    "leave restored draft",
  );
  await click(".q-dialog button", "Salir");
  await waitFor("!location.pathname.endsWith('/edit')", "leave restored form");
  await evaluate(
    `document.querySelector('.q-page').__vueParentComponent.appContext.config.globalProperties.$router.push('/client/change-requests/${requestId}/edit')`,
  );
  await waitFor(
    "location.pathname.endsWith('/edit') && !!document.querySelector('.q-banner') && !document.querySelector('.q-dialog') && !document.querySelector('.markdown-editor')",
    "cancelled request",
  );
  await check(
    "Cancelled request has no editable Markdown or save action",
    "!document.querySelector('.markdown-editor') && !document.querySelector('.form-actions button.bg-primary') && !document.querySelector('.q-dialog')",
  );
  await shot("cancelled-request-fixture-mobile");
  await check(
    "Client UI never requested private professional preferences",
    "!window.__requests.some(p=>p.startsWith('/api/v1/users/me'))",
  );
  await writeFile(
    join(output, "client-states-verification.json"),
    JSON.stringify(
      {
        checkedAt: new Date().toISOString(),
        source:
          "Browser fixtures only; real client credentials/token tests are in f89/dashboard-portal-verification.json",
        checks,
      },
      null,
      2,
    ),
  );
}
