import { readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { randomBytes } from "node:crypto";

// Real dedicated account and disposable records; failure responses are browser-only.
export async function verifyMigration({
  call,
  evaluate,
  navigate,
  waitFor,
  output,
  account,
}) {
  const checks = [],
    matrix = [],
    audits = [],
    created = { tags: [] };
  const check = async (label, expression) => {
    if (!(await evaluate(expression)))
      throw Error("Migration verification failed: " + label);
    if (!checks.includes(label)) checks.push(label);
    console.log("Verified migration: " + label);
  };
  const api = (path, method = "GET", body) =>
    evaluate(
      `fetch('/api/v1'+${JSON.stringify(path)},{method:${JSON.stringify(method)},headers:{'Content-Type':'application/json'},${body ? `body:JSON.stringify(${JSON.stringify(body)}),` : ""}}).then(async r=>{if(!r.ok)throw Error('Test API '+r.status);return r.json();})`,
    );
  const click = (selector, text) =>
    evaluate(
      `Array.from(document.querySelectorAll(${JSON.stringify(selector)})).find(e=>e.textContent.includes(${JSON.stringify(text)})).click()`,
    );
  const type = (selector, value) =>
    evaluate(
      `(()=>{const e=document.querySelector(${JSON.stringify(selector)});e.value=${JSON.stringify(value)};e.dispatchEvent(new Event('input',{bubbles:true}));})()`,
    );
  const shot = async (name) => {
    const { data } = await call("Page.captureScreenshot", {
      format: "png",
      captureBeyondViewport: false,
    });
    await writeFile(join(output, name + ".png"), Buffer.from(data, "base64"));
  };
  const viewport = async (width, height = 1000, scale = 1) => {
    await call("Emulation.setDeviceMetricsOverride", {
      width,
      height,
      deviceScaleFactor: scale,
      mobile: width < 600,
    });
    await evaluate("new Promise(r=>setTimeout(r,300))");
  };
  const theme = async (id, mode, density) => {
    await evaluate(
      `import('/src/stores/theme.ts').then(m=>m.useThemeStore().preview({...m.useThemeStore().appearance,theme_id:${JSON.stringify(id)},mode:${JSON.stringify(mode)},density:${JSON.stringify(density)},reduced_motion:true}))`,
    );
    await evaluate("new Promise(r=>setTimeout(r,300))");
  };
  const audit = async (label) => {
    const { nodes } = await call("Accessibility.getFullAXTree");
    const nameless = nodes
      .filter(
        (n) =>
          !n.ignored &&
          [
            "button",
            "link",
            "checkbox",
            "combobox",
            "textbox",
            "tab",
            "radio",
          ].includes(n.role?.value) &&
          !n.name?.value?.trim() &&
          !n.properties?.some((p) => p.name === "disabled" && p.value.value),
      )
      .map((n) => ({ role: n.role.value, node: n.backendDOMNodeId }));
    const contrast = await evaluate(`(()=>{
      const rgb=s=>{const a=s.match(/[\\d.]+/g)?.map(Number);return a?.length>=3?a:null;};
      const lum=c=>{const x=c.slice(0,3).map(v=>{v/=255;return v<=.04045?v/12.92:((v+.055)/1.055)**2.4;});return x[0]*.2126+x[1]*.7152+x[2]*.0722;};
      const ratio=(a,b)=>{const x=lum(a),y=lum(b);return(Math.max(x,y)+.05)/(Math.min(x,y)+.05);};
      const walker=document.createTreeWalker(document.querySelector('.q-dialog .q-card') ?? document.querySelector('.q-page'),NodeFilter.SHOW_TEXT);const failures=[];let node;
      while(node=walker.nextNode()){
        const el=node.parentElement,text=node.textContent.trim();if(!text||!el||el.closest('.q-icon,svg,canvas,script,style,[disabled],.disabled,.q-field--disabled'))continue;
        const r=el.getBoundingClientRect();if(!el.checkVisibility({visibilityProperty:true,opacityProperty:true})||!r.width||!r.height||r.bottom<0||r.top>innerHeight||r.right<0||r.left>innerWidth)continue;
        const style=getComputedStyle(el),fg=rgb(style.color);if(!fg)continue;
        let bg=[255,255,255],parent=el,layers=[],transparent=false;
        while(parent){const s=getComputedStyle(parent);if(Number(s.opacity)<.99){transparent=true;break;}const c=rgb(s.backgroundColor);if(c&&(c[3]??1)>0)layers.push(c);if(c&&(c[3]??1)===1)break;parent=parent.parentElement;}
        if(transparent)continue;
        for(const c of layers.reverse()){const alpha=c[3]??1;bg=c.slice(0,3).map((v,i)=>v*alpha+bg[i]*(1-alpha));}
        const large=parseFloat(style.fontSize)>=24||(parseFloat(style.fontSize)>=18.66&&Number(style.fontWeight)>=700),minimum=large?3:4.5,value=ratio(fg,bg);
        if(value<minimum-.05)failures.push({text:text.slice(0,90),selector:el.tagName+'.'+el.className,color:style.color,background:bg,ratio:Number(value.toFixed(2)),minimum});
      }
      return failures;
    })()`);
    audits.push({ label, nameless, contrast });
    await writeFile(
      join(output, "audit-progress.json"),
      JSON.stringify({ checks, matrix, audits }, null, 2),
    );
    if (nameless.length || contrast.length)
      console.log(
        "Audit findings: " +
          label +
          " names=" +
          nameless.length +
          " contrast=" +
          contrast.length,
      );
  };
  await check(
    "Dedicated migration account",
    "fetch('/api/v1/users/me').then(r=>r.json()).then(u=>u.email==='ui-tests@horbyta.test'&&u.role==='admin')",
  );
  const user = await api("/users/me");
  const table = await evaluate(
    "fetch('/api/v1/users/me/ui-state/datatable/tasks_table').then(r=>r.ok?r.json():null)",
  );
  let changedPreferences = false,
    changedTable = false,
    otherUserId = null;
  try {
    created.client = (
      await api("/clients", "POST", {
        name: "[UI] Revisión integral con nombre largo de cliente",
        email: "migration@horbyta.test",
      })
    ).id;
    created.project = (
      await api("/projects", "POST", {
        name: "[UI] Revisión integral de productividad y experiencia de usuario con título largo",
        client_id: created.client,
        billable: true,
        project_billing: {
          currency_type: "USD",
          billing_type: "fixed",
          fixed_price: "1234.50",
        },
        description:
          "## Objetivos\n\nComprobar **contraste**, foco y reflow.\n\n```text\n" +
          "contenido_largo_".repeat(18) +
          "\n```",
      })
    ).id;
    if (process.env.UI_MIGRATION_RESUME !== "1")
      created.milestone = (
        await api("/milestones", "POST", {
          project_id: created.project,
          name: "[UI] Entrega y revisión de accesibilidad",
          status: "open",
          description: "Revisar **objetivos**.",
          due_date: "2026-11-01T12:00:00Z",
        })
      ).id;
    for (const [index, color] of [
      "#FFFFFF",
      "#e44d26",
      "#242943",
      "#75D5AC",
    ].entries())
      created.tags.push(
        (
          await api("/tags", "POST", {
            name: "[UI] Color de revisión " + index,
            color,
          })
        ).id,
      );
    created.task = (
      await api("/tasks", "POST", {
        title:
          "[UI] Una tarea con título largo para comprobar lectura, acciones y preferencias en cualquier tamaño",
        project_id: created.project,
        milestone_id: created.milestone,
        status: "todo",
        priority: 3,
        complexity: 2,
        block_parent: false,
        description:
          "## Trabajo pendiente\n\nRevisar **accesibilidad** y contraste.",
        tag_ids: created.tags,
      })
    ).id;
    if (process.env.UI_MIGRATION_RESUME !== "1")
      created.invoice = (
        await api("/invoices", "POST", {
          project_id: created.project,
          amount: "1234.50",
          provider: "manual",
          time_entry_ids: [],
          freelancer_notes: "[UI] Solo prueba temporal",
        })
      ).id;
    if (process.env.UI_MIGRATION_RESUME !== "1")
      created.payment = (
        await api("/payments", "POST", {
          project_id: created.project,
          amount: "50.25",
          notes: "[UI] Solo prueba temporal",
        })
      ).id;
    if (process.env.UI_MIGRATION_RESUME !== "1")
      created.request = (
        await api("/change-requests", "POST", {
          project_id: created.project,
          client_id: created.client,
          title: "[UI] Revisar diseño del espacio de trabajo",
          client_request_description:
            "## Solicitud\n\nComprobar **legibilidad y foco**.",
          impact_hours: "0",
          impact_cost: "0",
          schedule_impact_days: 0,
        })
      ).id;
    if (process.env.UI_MIGRATION_RESUME === "1") {
      const saved = JSON.parse(
        await readFile(join(output, "audit-progress.json"), "utf8"),
      );
      if (
        saved.matrix.length !== 96 ||
        saved.audits.length < 81 ||
        saved.audits.some((a) => a.nameless.length || a.contrast.length)
      )
        throw Error("Cannot resume an incomplete or failing appearance audit");
      for (const label of saved.checks)
        if (!checks.includes(label)) checks.push(label);
      matrix.push(...saved.matrix);
      audits.push(...saved.audits);
      console.log(
        "Reused approved appearance matrix and page-family audit; only interaction/storage/account checks remain.",
      );
    } else {
      // Full matrix on shared list/form components, without reloading the API for each theme.
      for (const route of ["/tasks", "/tasks/new"]) {
        await viewport(1440);
        await navigate(route);
        for (const id of ["orbit", "linen", "forest"])
          for (const mode of ["light", "dark"])
            for (const density of ["comfortable", "compact"]) {
              await theme(id, mode, density);
              for (const width of [320, 360, 768, 1440]) {
                await viewport(width);
                const state = await evaluate(
                  "({width:innerWidth,scroll:document.documentElement.scrollWidth,id:document.documentElement.dataset.themeId,mode:document.documentElement.dataset.theme,density:document.documentElement.dataset.density})",
                );
                if (state.scroll > state.width + 1)
                  throw Error(
                    "Matrix overflow " +
                      JSON.stringify({
                        route,
                        id,
                        mode,
                        density,
                        width,
                        state,
                      }),
                  );
                matrix.push({ route, id, mode, density, width, passed: true });
                if (width === 360 || width === 1440)
                  await audit([route, id, mode, density, width].join(" "));
              }
              if (density === "comfortable")
                await shot(
                  "matrix-" +
                    route.replaceAll("/", "-") +
                    "-" +
                    id +
                    "-" +
                    mode,
                );
            }
      }
      checks.push("96 list/form theme-mode-density-viewport combinations");
      // Every page family, including real details and edits, gets a narrow-screen review.
      const routes = [
        "/",
        "/projects",
        "/clients",
        "/milestones",
        "/invoices",
        "/payments",
        "/change-requests",
        "/categories",
        "/activities",
        "/tags",
        "/admin/users",
        "/board",
        "/settings",
        ...[
          "projects",
          "clients",
          "milestones",
          "invoices",
          "payments",
          "change-requests",
        ].flatMap((f) => [
          "/" + f + "/new",
          "/" +
            f +
            "/" +
            created[
              {
                projects: "project",
                clients: "client",
                milestones: "milestone",
                invoices: "invoice",
                payments: "payment",
                "change-requests": "request",
              }[f]
            ],
          "/" +
            f +
            "/" +
            created[
              {
                projects: "project",
                clients: "client",
                milestones: "milestone",
                invoices: "invoice",
                payments: "payment",
                "change-requests": "request",
              }[f]
            ] +
            "/edit",
        ]),
        "/tasks/" + created.task,
        "/tasks/" + created.task + "/edit",
      ];
      for (const route of routes) {
        await navigate(route);
        await viewport(360);
        await theme("forest", "dark", "compact");
        await check(
          "Page family reflow " + route.replace(/[0-9a-f-]{36}/g, "fixture"),
          "document.documentElement.scrollWidth<=innerWidth+1 && !!document.querySelector('.q-page')",
        );
        await audit(route.replace(/[0-9a-f-]{36}/g, "fixture"));
        if (
          route.endsWith("/new") ||
          route === "/board" ||
          route === "/settings" ||
          route === "/tags"
        )
          await shot(
            "family-" +
              route.replace(/[0-9a-f-]{36}/g, "fixture").replaceAll("/", "-"),
          );
      }
    }
    await navigate("/tasks/" + created.task + "/edit");
    await check(
      "User colors retain legible tag text",
      "Array.from(document.querySelectorAll('.q-chip')).filter(c=>c.textContent.includes('[UI] Color')).length===4",
    );
    await audit("real task tag colors");
    await navigate("/tasks/new");
    await viewport(720, 500, 2);
    await check(
      "200 percent desktop spatial reflow",
      "document.documentElement.scrollWidth<=innerWidth+1 && !!document.querySelector('.form-actions')",
    );
    await evaluate(
      "document.querySelector('.markdown-editor textarea').focus()",
    );
    await check(
      "Focused Markdown stays above sticky actions",
      "(()=>{const a=document.activeElement.getBoundingClientRect(),b=document.querySelector('.form-actions').getBoundingClientRect();return a.top>=60 && a.top<b.top;})()",
    );
    await shot("form-200-percent-reflow");
    await viewport(1440);
    await navigate("/tags");
    await click(".dt-actions button", "Nuevo");
    await waitFor("!!document.querySelector('.q-dialog input')", "tag dialog");
    await audit("tag dialog");
    await evaluate(
      "document.querySelector('.q-dialog button[aria-label=\"Cerrar\"]').click()",
    );
    await waitFor("!document.querySelector('.q-dialog')", "tag close");
    await check(
      "Tag dialog returns focus to create action",
      "document.activeElement.textContent.includes('Nuevo')",
    );
    await navigate("/tasks");
    await viewport(360);
    await call("Emulation.setTouchEmulationEnabled", { enabled: true });
    await evaluate(
      "document.querySelector('button[aria-label=\"Abrir registro\"]').scrollIntoView({block:'center',inline:'center'})",
    );
    await evaluate(
      "new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)))",
    );
    const point = await evaluate(
      "(()=>{const r=document.querySelector('button[aria-label=\"Abrir registro\"]').getBoundingClientRect();return{x:r.x+r.width/2,y:r.y+r.height/2,radiusX:1,radiusY:1,force:1};})()",
    );
    await call("Input.dispatchTouchEvent", {
      type: "touchStart",
      touchPoints: [point],
    });
    await evaluate("new Promise(r=>setTimeout(r,80))");
    await call("Input.dispatchTouchEvent", {
      type: "touchEnd",
      touchPoints: [],
    });
    try {
      await waitFor(
        "location.pathname.startsWith('/tasks/') && !!document.querySelector('.detail-page')",
        "touch open record",
      );
    } catch (error) {
      await shot("touch-failure");
      await writeFile(
        join(output, "touch-failure.json"),
        JSON.stringify(
          {
            point,
            state: await evaluate(
              "({path:location.pathname,underPointer:document.elementFromPoint(" +
                point.x +
                "," +
                point.y +
                ")?.outerHTML.slice(0,400)})",
            ),
          },
          null,
          2,
        ),
      );
      throw error;
    }
    checks.push("Touch opens a record with one explicit action");
    await call("Emulation.setTouchEmulationEnabled", { enabled: false });
    await navigate("/tasks");
    await viewport(1440);
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
      "!!document.activeElement.closest('.q-dialog')",
      "keyboard columns focused",
    );
    await evaluate("new Promise(r=>setTimeout(r,300))");
    const focused = [];
    for (let index = 0; index < 20; index++) {
      await call("Input.dispatchKeyEvent", {
        type: "keyDown",
        key: "Tab",
        code: "Tab",
        windowsVirtualKeyCode: 9,
      });
      await call("Input.dispatchKeyEvent", {
        type: "keyUp",
        key: "Tab",
        code: "Tab",
        windowsVirtualKeyCode: 9,
      });
      await evaluate("new Promise(r=>setTimeout(r,50))");
      focused.push(
        await evaluate(
          "({inside:!!document.activeElement.closest('.q-dialog'),tag:document.activeElement.tagName})",
        ),
      );
    }
    await writeFile(
      join(output, "keyboard-focus.json"),
      JSON.stringify(focused, null, 2),
    );
    if (!focused.every((f) => f.inside))
      throw Error("Keyboard escaped modal focus");
    checks.push("Twenty Tab steps stay inside the modal");
    for (const type of ["keyDown", "keyUp"])
      await call("Input.dispatchKeyEvent", {
        type,
        key: "Escape",
        code: "Escape",
        windowsVirtualKeyCode: 27,
      });
    await waitFor("!document.querySelector('.q-dialog')", "keyboard close");
    await check(
      "Modal restores trigger focus",
      "document.activeElement.getAttribute('aria-label')==='Configurar columnas'",
    );
    for (const locale of ["es", "en", "pt"])
      for (const route of ["/settings", "/tasks", "/tasks/" + created.task]) {
        await navigate(route);
        await viewport(360);
        await evaluate(
          `import('/src/boot/i18n.ts').then(m=>m.setAppLocale(${JSON.stringify(locale)}))`,
        );
        await check(
          "Locale reflow " +
            locale +
            " " +
            route.replace(created.task, "fixture"),
          "document.documentElement.scrollWidth<=innerWidth+1",
        );
        await shot(
          "locale-" +
            locale +
            "-" +
            route.replace(created.task, "fixture").replaceAll("/", "-"),
        );
      }
    await evaluate("import('/src/boot/i18n.ts').then(m=>m.setAppLocale('es'))");
    await check(
      "Large negative amount respects regional display",
      "import('/src/utils/numberFormatting.ts').then(m=>{const s=m.formatNumberByPreferences('-987654321.75',{thousand_separator:'.',decimal_separator:',',decimal_quantity:2});return s.includes('987.654.321,75')&&s.includes('-');})",
    );
    // Storage errors cannot prevent a professional account from loading its server preferences.
    const blocked = await call("Page.addScriptToEvaluateOnNewDocument", {
      source:
        "for(const name of ['getItem','setItem']){const original=Storage.prototype[name];Storage.prototype[name]=function(key,...args){if(String(key).startsWith('horbyta:'))throw new DOMException('Blocked app storage','SecurityError');return original.call(this,key,...args);};}",
    });
    try {
      await navigate("/tasks");
      await check(
        "Blocked application storage still loads account and table",
        "document.querySelector('.q-table').textContent.includes('[UI]') && !!document.querySelector('.q-header')",
      );
    } finally {
      await call("Page.removeScriptToEvaluateOnNewDocument", {
        identifier: blocked.identifier,
      });
    }
    await navigate("/tasks");
    // Real A -> logout -> B -> logout -> A, with a temporary user that has no records.
    const other = {
      email: "migration-" + Date.now() + "@horbyta.test",
      password: "Ui9!" + randomBytes(24).toString("base64url"),
    };
    otherUserId = (
      await api("/users", "POST", {
        ...other,
        name: "[UI] Migration isolation",
        role: "user",
        status: "active",
      })
    ).id;
    await api("/users/me/preferences", "PATCH", {
      theme: "dark",
      appearance: {
        theme_id: "forest",
        density: "compact",
        reduced_motion: true,
      },
    });
    changedPreferences = true;
    await api("/users/me/ui-state/datatable/tasks_table", "PUT", {
      value: { visibleColumns: ["title"] },
    });
    changedTable = true;
    await navigate("/tasks");
    await api("/auth/logout", "POST");
    await navigate("/login");
    await type("input[type=email]", other.email);
    await type("input[type=password]", other.password);
    await evaluate("document.querySelector('form').requestSubmit()");
    await waitFor("location.pathname==='/'", "real B login");
    await navigate("/tasks");
    await check(
      "Real account B has independent appearance and empty table",
      "document.documentElement.dataset.themeId==='orbit' && document.documentElement.dataset.density==='comfortable' && document.querySelector('.empty-state')?.textContent.length>0",
    );
    await check(
      "Real account B cannot see account A table preferences",
      "fetch('/api/v1/users/me/ui-state?scope=datatable').then(r=>r.json()).then(v=>v.items.every(i=>i.key!=='tasks_table'||i.value.visibleColumns?.length!==1||i.value.visibleColumns[0]!=='title'))",
    );
    await api("/auth/logout", "POST");
    await navigate("/login");
    await type("input[type=email]", account.email);
    await type("input[type=password]", account.password);
    await evaluate("document.querySelector('form').requestSubmit()");
    await waitFor("location.pathname==='/'", "restore real A login");
    await navigate("/tasks");
    await check(
      "Real account A keeps appearance and table after B logout",
      "document.documentElement.dataset.themeId==='forest' && !Array.from(document.querySelectorAll('.q-table thead th')).some(th=>th.textContent.includes('Prioridad'))",
    );
    await viewport(1440);
    await writeFile(
      join(output, "migration-verification.json"),
      JSON.stringify(
        {
          checkedAt: new Date().toISOString(),
          resumedApprovedMatrix: process.env.UI_MIGRATION_RESUME === "1",
          checks,
          matrix,
          audits,
          scope:
            "Real dedicated account and disposable records; 200 percent spatial emulation, emulated touch, Chromium AX tree and computed text contrast; no screen reader or physical-device certification",
        },
        null,
        2,
      ),
    );
  } finally {
    // Restore A before deleting B; its password stays only in this process.
    await api("/auth/login", "POST", {
      email: account.email,
      password: account.password,
    });
    if (changedPreferences)
      await api("/users/me/preferences", "PATCH", {
        theme: user.preferences.theme,
        appearance: user.preferences.appearance ?? {
          theme_id: "orbit",
          density: "comfortable",
          reduced_motion: false,
        },
      });
    if (changedTable) {
      if (table)
        await api("/users/me/ui-state/datatable/tasks_table", "PUT", {
          value: table.value,
        });
      else await api("/users/me/ui-state/datatable/tasks_table", "DELETE");
    }
    if (otherUserId) await api("/users/" + otherUserId, "DELETE");
    if (created.request) {
      await api("/change-requests/" + created.request + "/cancel", "PATCH");
      await api("/change-requests/" + created.request, "DELETE");
    }
    if (created.payment) await api("/payments/" + created.payment, "DELETE");
    if (created.invoice) await api("/invoices/" + created.invoice, "DELETE");
    if (created.task) await api("/tasks/" + created.task, "DELETE");
    if (created.milestone)
      await api("/milestones/" + created.milestone, "DELETE");
    if (created.project) await api("/projects/" + created.project, "DELETE");
    if (created.client) await api("/clients/" + created.client, "DELETE");
    for (const id of created.tags) await api("/tags/" + id, "DELETE");
    await call("Emulation.setEmulatedMedia", { features: [] });
    await viewport(1440);
    await navigate("/");
  }
  const findings = audits.flatMap((a) =>
    a.nameless
      .map((f) => ({ ...f, label: a.label }))
      .concat(a.contrast.map((f) => ({ ...f, label: a.label }))),
  );
  if (findings.length)
    throw Error(
      "Migration audit found " +
        findings.length +
        " controls/contrast observations. Review migration-verification.json.",
    );
  console.log(
    "Migration checks passed: " +
      checks.length +
      "; matrix cells: " +
      matrix.length,
  );
}
