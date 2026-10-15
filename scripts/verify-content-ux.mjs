import assert from "node:assert/strict";
import { writeFile } from "node:fs/promises";
import { join } from "node:path";

// Demoras y contenido extremo se simulan en el navegador; no se escribe en la BD.
export async function verifyContentUx({
  call,
  evaluate,
  navigate,
  waitFor,
  output,
}) {
  const checks = [];
  const expect = async (expression, label) => {
    await waitFor(expression, label);
    checks.push(label);
  };
  const api = (path) =>
    evaluate(
      `fetch('/api/v1'+${JSON.stringify(path)},{credentials:'include'}).then(r=>r.json())`,
    );
  const q = encodeURIComponent(
    JSON.stringify({
      page: 1,
      limit: 50,
      order_by: null,
      order_asc: true,
      filters: [],
    }),
  );
  const projects = (await api(`/projects/search?q=${q}`)).items;
  const project = projects.find((p) => p.name === "Portal editorial Lumen");
  assert(project, "Se necesita el seed documental de desarrollo");
  const scope = await api(`/scope/projects/${project.id}`);
  if (process.env.UI_VERIFY_UX_ERRORS_ONLY !== "1") {
    const accepted = scope.revisions.find(
      (r) => r.revision === scope.accepted_revision,
    );
    accepted.document.exclusions =
      "# Exclusión con encabezado grande\n\nEste encabezado pertenece al documento del usuario.\n\n[Referencia externa](https://example.test)";
    accepted.document.deliverables =
      "# Entregable detallado\n\n## Segunda jerarquía\n\nDescripción de prueba.";
    const fixture = await call("Page.addScriptToEvaluateOnNewDocument", {
      source: `(() => {
    const original = window.fetch;
    window.fetch = (url, options) => String(url)==='/api/v1/scope/projects/${project.id}' && (!options?.method || options.method==='GET')
      ? Promise.resolve(new Response(JSON.stringify(${JSON.stringify(scope)}),{status:200,headers:{'Content-Type':'application/json'}}))
      : original(url, options);
  })();`,
    });
    await navigate(`/projects/${project.id}`);
    await expect(
      "Array.from(document.querySelectorAll('.scope-section__header h3')).some(h=>h.textContent==='Exclusiones')",
      "Sección del sistema identificable",
    );
    const structure = await evaluate(`(() => {
    const section=Array.from(document.querySelectorAll('.scope-section')).find(s=>s.querySelector('.scope-section__header h3')?.textContent==='Exclusiones');
    const header=section.querySelector('.scope-section__header');
    const body=section.querySelector('.scope-section__body');
    const link=body.querySelector('a');
    return {heading:body.querySelector('h1')?.textContent, backgrounds:[getComputedStyle(header).backgroundColor,getComputedStyle(section).backgroundColor],border:getComputedStyle(header).borderBottomWidth,underline:getComputedStyle(link).textDecorationLine,documents:document.querySelectorAll('.scope-content-document__header').length, actions:document.querySelectorAll('.scope-content-document__actions .q-btn--outline').length, middleDot:Array.from(document.querySelectorAll('.scope-document h2, .scope-section__header, .scope-document .q-btn')).some(el=>el.textContent.includes('·'))};
  })()`);
    assert.equal(structure.heading, "Exclusión con encabezado grande");
    assert.notEqual(...structure.backgrounds);
    assert.notEqual(structure.border, "0px");
    assert(structure.underline.includes("underline"));
    assert(structure.documents >= 3 && structure.actions >= 6);
    assert.equal(structure.middleDot, false);
    checks.push(
      "Markdown H1 dentro del cuerpo separado del encabezado del sistema",
      "Cabecera con superficie y borde distintos",
      "Enlaces subrayados y acciones con borde",
      "Tres documentos con metadatos y acciones independientes",
      "Sin separadores de punto medio en el alcance",
    );
    await evaluate("document.querySelector('.scope-document-index a').focus()");
    assert(
      await evaluate(
        "getComputedStyle(document.activeElement).outlineStyle!=='none'",
      ),
    );
    checks.push("Índice navegable con foco visible");
    for (const viewport of [
      { name: "desktop", width: 1440, height: 1000, mobile: false },
      { name: "mobile", width: 390, height: 844, mobile: true },
    ]) {
      await call("Emulation.setDeviceMetricsOverride", {
        ...viewport,
        deviceScaleFactor: 1,
      });
      await evaluate("window.scrollTo(0,0); new Promise(r=>setTimeout(r,300))");
      assert(
        await evaluate("document.documentElement.scrollWidth<=innerWidth+2"),
        `Sin desbordamiento ${viewport.name}`,
      );
      checks.push(`Alcance legible sin desbordamiento ${viewport.name}`);
      const { data } = await call("Page.captureScreenshot", {
        format: "png",
        captureBeyondViewport: true,
      });
      await writeFile(
        join(output, `scope-structure-${viewport.name}.png`),
        Buffer.from(data, "base64"),
      );
    }
    await call("Page.removeScriptToEvaluateOnNewDocument", {
      identifier: fixture.identifier,
    });
    await call("Emulation.setDeviceMetricsOverride", {
      width: 1440,
      height: 1000,
      mobile: false,
      deviceScaleFactor: 1,
    });
    const delay = await call("Page.addScriptToEvaluateOnNewDocument", {
      source: `(() => {
    const original=window.fetch;
    window.fetch=async(url,options)=>{
      if(String(url).startsWith('/api/v1/') && !String(url).includes('/auth/') && !String(url).includes('/users/me'))await new Promise(r=>setTimeout(r,1800));
      return original(url,options);
    };
  })()`,
    });
    const entries = [
      ["/", "dashboard"],
      ["/projects", "table"],
      ["/board", "board"],
      [`/projects/${project.id}`, "detail"],
      [`/projects/${project.id}/edit`, "form"],
      ["/projects/new", "form"],
      ["/tasks/new", "form"],
      ["/clients/new", "form"],
      ["/invoices/new", "form"],
      ["/payments/new", "form"],
      ["/change-requests/new", "form"],
      ["/settings", "form"],
    ];
    for (const [path, variant] of entries) {
      await call("Page.navigate", {
        url: new URL(path, "http://127.0.0.1:9107").href,
      });
      await expect(
        `!!document.querySelector('.content-skeleton--${variant}')`,
        `${path}: esqueleto ${variant} durante carga`,
      );
      assert(
        await evaluate(
          "!document.querySelector('.q-loading, .q-inner-loading')",
        ),
        `${path}: sin overlay de carga`,
      );
      assert(
        await evaluate(
          "document.querySelector('.content-skeleton').getAttribute('aria-busy')==='true'&&!!document.querySelector('.content-skeleton [aria-hidden=true]')",
        ),
        `${path}: semántica de carga`,
      );
      if (path === `/projects/${project.id}`) {
        const { data } = await call("Page.captureScreenshot", {
          format: "png",
        });
        await writeFile(
          join(output, "project-loading.png"),
          Buffer.from(data, "base64"),
        );
      }
      await expect(
        "!!document.querySelector('.q-page')&&!document.querySelector('.content-skeleton')",
        `${path}: contenido sustituye al esqueleto`,
      );
    }
    await call("Page.removeScriptToEvaluateOnNewDocument", {
      identifier: delay.identifier,
    });
  }
  const failure = await call("Page.addScriptToEvaluateOnNewDocument", {
    source: `(() => {
    const original=window.fetch;
    window.fetch=async(url,options)=>{
      if(String(url).startsWith('/api/v1/projects/search')){
        await new Promise(r=>setTimeout(r,1200));
        return new Response(JSON.stringify({error:'internal server error',type:'internal_server_error'}),{status:500,headers:{'Content-Type':'application/json'}});
      }
      return original(url,options);
    };
  })()`,
  });
  await call("Page.navigate", { url: "http://127.0.0.1:9107/projects" });
  await expect(
    "!!document.querySelector('.content-skeleton--table')",
    "Error simulado: esqueleto mientras espera",
  );
  await expect(
    "!!document.querySelector('.empty-state[role=alert]')&&!document.querySelector('.content-skeleton')",
    "Error simulado: alerta sustituye al esqueleto",
  );
  await call("Page.removeScriptToEvaluateOnNewDocument", {
    identifier: failure.identifier,
  });
  await navigate(`/projects/${project.id}`);
  await writeFile(
    join(output, "content-ux.json"),
    JSON.stringify({ checks }, null, 2),
  );
  console.log(
    `${checks.length} comprobaciones de jerarquía, enlaces y carga: OK`,
  );
}
