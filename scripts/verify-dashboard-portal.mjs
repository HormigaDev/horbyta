import { writeFile } from "node:fs/promises";
import { join } from "node:path";
import { randomBytes } from "node:crypto";

// Financial fixtures are browser responses, never inserted into the ledger.
// Portal fixtures belong only to the dedicated local test account and are removed.
export async function verifyDashboardPortal({
  call,
  evaluate,
  navigate,
  waitFor,
  output,
  account,
}) {
  const checks = [];
  const check = async (label, expression) => {
    if (!(await evaluate(expression)))
      throw Error(`Dashboard/portal verification failed: ${label}`);
    checks.push(label);
    console.log(`Verified: ${label}`);
  };
  const api = (path, method = "GET", body) =>
    evaluate(
      `fetch('/api/v1'+${JSON.stringify(path)},{method:${JSON.stringify(method)},headers:{'Content-Type':'application/json'},${body ? `body:JSON.stringify(${JSON.stringify(body)}),` : ""}}).then(async r=>{if(!r.ok)throw Error('Fixture API '+r.status); return r.json();})`,
    );
  const click = (selector, text) =>
    evaluate(
      `Array.from(document.querySelectorAll(${JSON.stringify(selector)})).find(el=>el.textContent.includes(${JSON.stringify(text)})).click()`,
    );
  const type = (selector, value) =>
    evaluate(
      `(()=>{const el=document.querySelector(${JSON.stringify(selector)});el.value=${JSON.stringify(value)};el.dispatchEvent(new Event('input',{bubbles:true}));el.dispatchEvent(new Event('change',{bubbles:true}));})()`,
    );
  const shot = async (name) => {
    await evaluate("document.fonts.ready.then(()=>true)");
    const { data } = await call("Page.captureScreenshot", {
      format: "png",
      captureBeyondViewport: false,
    });
    await writeFile(join(output, name + ".png"), Buffer.from(data, "base64"));
  };
  const viewport = (width) =>
    call("Emulation.setDeviceMetricsOverride", {
      width,
      height: 1000,
      deviceScaleFactor: 1,
      mobile: width < 600,
    });
  const chart = `document.querySelector('.echarts').__vueParentComponent.exposed.getOption()`;
  await check(
    "Dedicated local test account",
    "fetch('/api/v1/users/me').then(r=>r.json()).then(u=>u.email==='ui-tests@horbyta.test')",
  );
  const user = await api("/users/me");
  const fixture = {
    kpis: {
      scope_creep_recovered: [
        { currency_type: "USD", amount: "25" },
        { currency_type: "EUR", amount: "20" },
      ],
      accounts_receivable_total: [{ currency_type: "USD", amount: "0" }],
      overdue_tasks_total: 2,
    },
    monthly_revenue: [
      { month_date: "2026-11-01", currency_type: "USD", total_revenue: "12" },
      { month_date: "2026-02-01", currency_type: "EUR", total_revenue: "7" },
      { month_date: "2026-01-01", currency_type: "USD", total_revenue: "0" },
      { month_date: "2026-02-01", currency_type: "USD", total_revenue: "3" },
    ],
    time_by_activity: [
      {
        activity_name: "Diseño",
        billable: true,
        week_date: "2026-01-05",
        total_hours: "2",
      },
      {
        activity_name: "Diseño",
        billable: false,
        week_date: "2026-01-12",
        total_hours: "1",
      },
      {
        activity_name: "Revisión",
        billable: true,
        week_date: "2026-01-05",
        total_hours: "4",
      },
    ],
    billable_ratio_daily: [
      {
        work_date: "2026-11-01",
        billable_hours: "2",
        no_billable_hours: "1",
        total_hours: "3",
      },
      {
        work_date: "2026-01-01",
        billable_hours: "0",
        no_billable_hours: "0",
        total_hours: "0",
      },
    ],
    scope_creep_value: [],
    task_bottlenecks: [],
    accounts_receivable: [],
  };
  const injected = await call("Page.addScriptToEvaluateOnNewDocument", {
    source: `(()=>{const real=window.fetch.bind(window);window.__fixture=${JSON.stringify(fixture)};window.fetch=(input,options)=>{const url=new URL(typeof input==='string'?input:input.url,location.origin);if(url.pathname==='/api/v1/dashboard'){if(window.__dashboardFail)return Promise.resolve(new Response('{}',{status:500}));return Promise.resolve(Response.json(window.__fixture));}if(url.pathname.startsWith('/api/v1/dashboard/charts/')){if(window.__filterFail)return Promise.resolve(new Response('{}',{status:500}));const key={'monthly-revenue':'monthly_revenue','time-by-activity':'time_by_activity','billable-ratio':'billable_ratio_daily'}[url.pathname.split('/').pop()];return Promise.resolve(Response.json(window.__fixture[key]??[]));}return real(input,options);};})()`,
  });
  try {
    await navigate("/");
    await waitFor(
      "document.querySelectorAll('.echarts').length===3 && !!document.querySelector('.chart-data table')",
      "charts render",
    );
    await check(
      "Dashboard has summary, trends and follow-up",
      "document.querySelectorAll('.dashboard-section-heading').length===2 && document.querySelectorAll('.kpi-card').length===3",
    );
    await check(
      "Chronology uses raw dates before DD/MM formatting",
      `${chart}.xAxis[0].data.join('|')==='01/01/2026|01/02/2026|01/11/2026'`,
    );
    await check(
      "Currencies remain separate and missing points are zero",
      `(()=>{const s=${chart}.series;return s.length===2&&s.find(x=>x.name==='USD').data.join(',')==='0,3,12'&&s.find(x=>x.name==='EUR').data.join(',')==='0,7,0';})()`,
    );
    await check(
      "Data table retains observed zero and all currency rows",
      "document.querySelector('.chart-data tbody').children.length===6 && document.querySelector('.chart-data tbody tr').textContent.includes('EUR')",
    );
    await check(
      "Canvas has accessible description and series patterns",
      `${chart}.aria.enabled===true && ${chart}.aria.decal.show===true && !!document.querySelector('.echarts[aria-label]')`,
    );
    const initialColor = await evaluate(
      `${chart}.series.find(s=>s.name==='USD').itemStyle.color`,
    );
    await evaluate(
      "import('/src/stores/theme.ts').then(m=>m.useThemeStore().preview({...m.useThemeStore().appearance,theme_id:'forest'}))",
    );
    await waitFor(
      `${chart}.series.find(s=>s.name==='USD').itemStyle.color!==${JSON.stringify(initialColor)}`,
      "palette identity reacts",
    );
    await check(
      "Changing identity in same mode updates chart color",
      `${chart}.series.find(s=>s.name==='USD').itemStyle.color!==${JSON.stringify(initialColor)}`,
    );
    await evaluate(
      "import('/src/stores/theme.ts').then(m=>m.useThemeStore().preview({...m.useThemeStore().appearance,reduced_motion:true}))",
    );
    await waitFor(`${chart}.animation===false`, "motion preference");
    await check(
      "Reduced motion disables chart animation",
      `${chart}.animation===false`,
    );
    await evaluate(
      "import('/src/stores/userPreferences.ts').then(m=>{m.useUserPreferencesStore().dateFormat='MM/DD/YYYY';})",
    );
    await check(
      "Chronology survives MM/DD formatting",
      `${chart}.xAxis[0].data.join('|')==='01/01/2026|02/01/2026|11/01/2026'`,
    );
    await evaluate("window.__fixture.monthly_revenue[0].total_revenue='99'");
    await click(".page-header button", "Recargar");
    await waitFor(
      `${chart}.series.find(s=>s.name==='USD').data[2]===99`,
      "overview refresh updates charts",
    );
    await check(
      "Refresh propagates new overview rows into cards",
      `${chart}.series.find(s=>s.name==='USD').data[2]===99`,
    );
    await evaluate("document.querySelector('.chart-data summary').click()");
    await shot("dashboard-fixture-desktop");
    await evaluate(
      "document.querySelector('.chart-card button[aria-label]').click()",
    );
    await waitFor(
      "!!document.querySelector('.filter-dialog')",
      "chart filter dialog",
    );
    await evaluate(
      "Array.from(document.querySelectorAll('.filter-dialog .q-field')).find(f=>f.querySelector('.q-field__label')?.textContent==='Moneda').click()",
    );
    await waitFor(
      "!!document.querySelector('.q-menu [role=option]')",
      "currency options",
    );
    await click(".q-menu [role=option]", "USD");
    await evaluate("window.__filterFail=true");
    await click(".filter-dialog button", "Aplicar");
    await waitFor(
      "!!document.querySelector('.filter-dialog [role=alert]')",
      "failed filter",
    );
    await check(
      "Failed filter retains dialog, draft and confirmed data",
      `!!document.querySelector('.filter-dialog') && !document.querySelector('.chart-card .q-badge') && ${chart}.series.find(s=>s.name==='USD').data[2]===99`,
    );
    // The shared error notifier may add a second dialog; close only that dialog.
    await evaluate(
      "Array.from(document.querySelectorAll('.q-dialog button')).filter(b=>!b.closest('.filter-dialog')).forEach(b=>b.click())",
    );
    await evaluate("window.__filterFail=false");
    await click(".filter-dialog button", "Aplicar");
    await waitFor(
      "!document.querySelector('.filter-dialog')",
      "successful retry",
    );
    await check(
      "Successful filter is confirmed only after response",
      "!!document.querySelector('.chart-card .q-badge')",
    );
    await viewport(360);
    await check(
      "Dashboard and chart tables do not overflow mobile",
      "document.documentElement.scrollWidth<=innerWidth",
    );
    await evaluate(
      "document.querySelectorAll('.q-notification button:last-child').forEach(b=>b.click())",
    );
    await shot("dashboard-fixture-mobile");
    await evaluate(
      "window.scrollTo(0,document.querySelector('.dashboard-section-heading').getBoundingClientRect().top+scrollY-90)",
    );
    await shot("dashboard-charts-mobile");
    // Empty data and retry states are tested through browser-only responses.
    await navigate("/");
    await evaluate(
      "Object.keys(window.__fixture).filter(k=>Array.isArray(window.__fixture[k])).forEach(k=>window.__fixture[k]=[])",
    );
    await click(".page-header button", "Recargar");
    await waitFor(
      "document.querySelectorAll('.echarts').length===0",
      "empty charts",
    );
    await check(
      "Empty charts have an explicit state",
      "document.querySelectorAll('.chart-empty').length>=3",
    );
    await evaluate("window.__dashboardFail=true");
    await click(".page-header button", "Recargar");
    await waitFor(
      "!!document.querySelector('.empty-state')",
      "overview failure",
    );
    await check(
      "Refresh failure keeps previous summary and retry action",
      "!!document.querySelector('.empty-state button') && document.querySelectorAll('.kpi-card').length===3",
    );
  } finally {
    await call("Page.removeScriptToEvaluateOnNewDocument", {
      identifier: injected.identifier,
    });
  }
  await viewport(1440);
  await navigate("/");
  let clientId, projectId, requestId;
  const clientPassword = "Ui9!" + randomBytes(24).toString("base64url");
  const email = "portal-ui-" + Date.now() + "@horbyta.test";
  const loginPath = "/client/u/" + user.id + "/login";
  const tracker = await call("Page.addScriptToEvaluateOnNewDocument", {
    source: `(()=>{const real=window.fetch.bind(window);window.__portalRequests=[];window.fetch=(input,options)=>{window.__portalRequests.push(new URL(typeof input==='string'?input:input.url,location.origin).pathname);return real(input,options);};})()`,
  });
  try {
    clientId = (
      await api("/clients", "POST", { name: "[UI] Portal de pruebas", email })
    ).id;
    await api("/clients/" + clientId + "/auth/credentials", "POST", {
      email,
      password: clientPassword,
    });
    projectId = (
      await api("/projects", "POST", {
        name: "[UI] Entrega y revisión de un proyecto con título largo",
        description:
          "## Próxima entrega\n\nRevisar **diseño** y confirmar los objetivos.",
        client_id: clientId,
        billable: false,
        milestones: [
          {
            name: "[UI] Revisar primera entrega",
            due_date: "2026-11-01T12:00:00Z",
          },
        ],
      })
    ).id;
    await navigate(loginPath);
    await check(
      "Portal login supports password managers and initial focus",
      "document.querySelector('input[type=email]').autocomplete==='username' && document.querySelector('input[type=password]').autocomplete==='current-password' && document.activeElement.type==='email'",
    );
    await shot("portal-login-desktop");
    await type("input[type=email]", email);
    await type("input[type=password]", clientPassword);
    await evaluate("document.querySelector('form').requestSubmit()");
    await waitFor(
      "location.pathname==='/client' && !!document.querySelector('.kpi-card')",
      "real client credentials login",
    );
    await check(
      "Real credentials enter the client dashboard",
      "document.querySelector('.q-header').textContent.includes('Portal de pruebas')",
    );
    await check(
      "Portal does not request professional preferences",
      "!window.__portalRequests.some(p=>p.startsWith('/api/v1/users/me'))",
    );
    await evaluate(
      "document.querySelector('button[aria-label=\"Apariencia\"]').click()",
    );
    await waitFor(
      "!!document.querySelector('.appearance-menu')",
      "portal theme menu",
    );
    await click(".theme-menu-options button", "Bosque");
    await evaluate(
      "import('/src/stores/theme.ts').then(m=>m.useThemeStore().commit({...m.useThemeStore().appearance,mode:'dark',density:'compact',reduced_motion:true}))",
    );
    await evaluate(
      "document.querySelector('.q-menu').__vueParentComponent?.proxy;document.body.click()",
    );
    await navigate("/client");
    await check(
      "Portal appearance survives reload independently",
      "document.documentElement.dataset.themeId==='forest' && document.documentElement.dataset.theme==='dark' && document.documentElement.dataset.density==='compact'",
    );
    await check(
      "Theme color follows resolved navigation surface",
      "document.querySelector('meta[name=theme-color]').content===getComputedStyle(document.documentElement).getPropertyValue('--surface-nav').trim()",
    );
    await shot("portal-dashboard-desktop");
    await navigate("/client/projects");
    await check(
      "Portal project list exposes actual linked project",
      "document.querySelector('.q-table').textContent.includes('Entrega y revisión')",
    );
    await evaluate(
      `import('/src/stores/datatable.ts').then(m=>m.useDataTableStore().saveTableState('client_projects_table',{pagination:{sortBy:'name',descending:true,page:1,rowsPerPage:10}}))`,
    );
    await navigate("/client/projects");
    await check(
      "Portal table preferences persist in client-scoped storage",
      `JSON.parse(localStorage.getItem('horbyta:client-table:v1:${user.id}:${clientId}:client_projects_table')).value.pagination.rowsPerPage===10`,
    );
    await check(
      "Portal tables never call professional UI-state API",
      "!window.__portalRequests.some(p=>p.startsWith('/api/v1/users/me'))",
    );
    await navigate("/client/projects/" + projectId);
    await check(
      "Project detail renders semantic Markdown",
      "document.querySelector('.detail-page').textContent.includes('Próxima entrega')",
    );
    await shot("portal-project-desktop");
    await navigate("/client/milestones?project_id=" + projectId);
    await check(
      "Portal milestone list shows linked milestone",
      "document.querySelector('.q-table').textContent.includes('Revisar primera entrega')",
    );
    await navigate("/client/change-requests/new");
    await check(
      "Client request form has shared actions and preserved permission flow",
      "!!document.querySelector('.form-actions') && !!document.querySelector('.markdown-editor')",
    );
    await shot("portal-request-desktop");
    const request = await api("/change-requests/client", "POST", {
      project_id: projectId,
      title: "[UI] Revisar accesibilidad del portal",
      client_request_description:
        "## Revisión solicitada\n\nComprobar contraste, foco y **legibilidad**.",
    });
    requestId = request.id;
    await navigate("/client/change-requests/" + requestId);
    await check(
      "Client request detail renders real submitted Markdown",
      "!!document.querySelector('.detail-page h1') && document.querySelector('.detail-page').textContent.includes('Revisión solicitada')",
    );
    await shot("portal-request-detail-desktop");
    await navigate("/client/change-requests/" + requestId + "/edit");
    await check(
      "Submitted request remains editable",
      "!!document.querySelector('.form-actions button.bg-primary')",
    );
    await api("/change-requests/client/" + requestId + "/cancel", "PATCH", {
      version: request.version,
    });
    await navigate("/client/change-requests/" + requestId + "/edit");
    await check(
      "Cancelled request preserves status lock",
      "!document.querySelector('.form-actions button.bg-primary') && !!document.querySelector('.q-banner')",
    );
    const milestones = await api(
      "/milestones/client/search-by-project/" +
        projectId +
        "?q=" +
        encodeURIComponent(
          JSON.stringify({
            page: 1,
            limit: 10,
            order_by: "created_at",
            order_asc: false,
            filters: [],
          }),
        ),
    );
    const milestoneId = milestones.items?.[0]?.id;
    if (!milestoneId) throw Error("Fixture milestone missing");
    await navigate(
      "/client/milestones/" + milestoneId + "?project_id=" + projectId,
    );
    await check(
      "Client milestone detail has heading and linked project",
      "!!document.querySelector('.detail-page h1') && document.querySelector('.detail-page').textContent.includes('Revisar primera entrega')",
    );
    await shot("portal-milestone-desktop");
    await viewport(360);
    for (const path of [
      "/client",
      "/client/projects",
      "/client/projects/" + projectId,
      "/client/change-requests",
      "/client/change-requests/new",
    ]) {
      await navigate(path);
      await check(
        "Portal responsive " + path.replace(projectId, "fixture"),
        "document.documentElement.scrollWidth<=innerWidth",
      );
      await shot(
        "portal-mobile-" +
          path.replace(projectId, "fixture").replaceAll("/", "-"),
      );
    }
    await navigate("/client");
    await check(
      "Portal changes leave professional preferences untouched",
      `fetch('/api/v1/users/me').then(r=>r.json()).then(u=>JSON.stringify(u.preferences)===${JSON.stringify(JSON.stringify(user.preferences))})`,
    );
    // The backend supports either credentials or token; enable the token after credentials testing.
    const magic = await api(
      "/clients/" + clientId + "/auth/magic-token",
      "POST",
    );
    await api("/auth/logout-client", "POST");
    await navigate(loginPath);
    await click("[role=tab]", "Token");
    await waitFor("!!document.querySelector('textarea')", "magic token form");
    await type("textarea", magic.magic_client_token);
    await evaluate(
      "document.querySelector('textarea').closest('form').requestSubmit()",
    );
    await waitFor("location.pathname==='/client'", "real magic token login");
    await check(
      "Real magic token flow still enters portal",
      "!!document.querySelector('.q-header')",
    );
    await api("/auth/logout-client", "POST");
  } finally {
    await call("Page.removeScriptToEvaluateOnNewDocument", {
      identifier: tracker.identifier,
    });
    if (requestId) {
      const current = await api("/change-requests/" + requestId);
      if (current.status !== "cancelled")
        await api("/change-requests/" + requestId + "/cancel", "PATCH");
      await api("/change-requests/" + requestId, "DELETE");
    }
    if (projectId) await api("/projects/" + projectId, "DELETE");
    if (clientId) await api("/clients/" + clientId, "DELETE");
  }
  await api("/auth/logout", "POST");
  await call("Network.setBlockedURLs", {
    urls: ["*fonts.googleapis.com*", "*fonts.gstatic.com*"],
  });
  await navigate("/login");
  await check(
    "Professional login uses correct autofill and focus",
    "document.querySelector('input[type=email]').autocomplete==='username'&&document.querySelector('input[type=password]').autocomplete==='current-password'&&document.activeElement.type==='email'",
  );
  for (const width of [1440, 360]) {
    await viewport(width);
    for (const locale of ["es", "en", "pt"]) {
      await evaluate(
        `import('/src/boot/i18n.ts').then(m=>m.setAppLocale(${JSON.stringify(locale)}))`,
      );
      await check(
        "Login without external fonts " + locale + " " + width,
        "document.documentElement.scrollWidth<=innerWidth && !!document.querySelector('.auth-card h2')",
      );
      await shot("login-offline-font-" + locale + "-" + width);
    }
  }
  await evaluate("import('/src/boot/i18n.ts').then(m=>m.setAppLocale('es'))");
  // Mock only the UI challenge/errors; production 2FA backend is not changed.
  await evaluate(
    `(()=>{const real=window.fetch.bind(window);window.__loginBodies=[];window.fetch=(input,opts)=>{if(input==='/api/v1/auth/login'){window.__loginBodies.push(JSON.parse(opts.body));return Promise.resolve(Response.json({type:window.__loginBodies.length===1?'two_factor_auth_code_required':'invalid_two_factor_auth_code',error:'Test challenge'},{status:401}));}return real(input,opts);};window.__restoreFetch=()=>window.fetch=real;})()`,
  );
  await type("input[type=email]", "challenge@horbyta.test");
  await type("input[type=password]", "Test-password-123");
  await evaluate("document.querySelector('form').requestSubmit()");
  await waitFor(
    "!!document.querySelector('input[autocomplete=\"one-time-code\"]')",
    "2FA challenge",
  );
  await check(
    "2FA input has autofill and receives focus",
    "document.activeElement.autocomplete==='one-time-code' && document.activeElement.inputMode==='numeric'",
  );
  await type('input[autocomplete="one-time-code"]', "123456");
  await evaluate("document.querySelector('form').requestSubmit()");
  await waitFor(
    "!!document.querySelector('[role=alert]')",
    "invalid 2FA feedback",
  );
  await check(
    "2FA sends code and exposes recoverable error",
    "window.__loginBodies[1].two_factor_auth_code==='123456' && !window.__loginBodies[1].two_factor_auth_backup_code && document.querySelector('[role=alert]').textContent.length>0",
  );
  await click("[role=tab]", "Respaldo");
  await waitFor(
    "!!document.querySelector('input[name=backup-code]')",
    "backup code",
  );
  await type("input[name=backup-code]", "backup-test-code");
  await evaluate("document.querySelector('form').requestSubmit()");
  await waitFor("window.__loginBodies.length===3", "backup request");
  await check(
    "Backup code preserves separate 2FA contract",
    "window.__loginBodies[2].two_factor_auth_backup_code==='backup-test-code' && !window.__loginBodies[2].two_factor_auth_code",
  );
  await shot("login-two-factor-mobile");
  await evaluate("window.__restoreFetch()");
  await navigate("/register");
  await check(
    "Registration supports name and new-password autofill",
    "!!document.querySelector('input[autocomplete=name]') && document.querySelectorAll('input[autocomplete=new-password]').length===2",
  );
  await check(
    "Registration fits mobile",
    "document.documentElement.scrollWidth<=innerWidth",
  );
  await shot("register-mobile");
  await navigate("/unknown-ui-test");
  await check(
    "404 has readable heading and return action",
    "!!document.querySelector('.not-found-card h1') && !!document.querySelector('.not-found-card a') && document.documentElement.scrollWidth<=innerWidth",
  );
  await shot("404-mobile");
  await call("Network.setBlockedURLs", { urls: [] });
  await viewport(1440);
  await navigate("/login");
  await type("input[type=email]", account.email);
  await type("input[type=password]", account.password);
  await evaluate("document.querySelector('form').requestSubmit()");
  await waitFor("location.pathname==='/'", "restore professional session");
  await writeFile(
    join(output, "dashboard-portal-verification.json"),
    JSON.stringify(
      {
        checkedAt: new Date().toISOString(),
        checks,
        financialData: "Browser-only fixtures",
        twoFactor: "Browser-only challenge/errors; backend unchanged",
        portal: "Real disposable client, project and milestone, cleaned up",
      },
      null,
      2,
    ),
  );
  console.log(`Dashboard/portal verification: ${checks.length} checks passed.`);
}
