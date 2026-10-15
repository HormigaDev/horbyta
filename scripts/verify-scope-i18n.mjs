import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import { createRequire } from "node:module";

const require = createRequire(
  new URL("../frontend/package.json", import.meta.url),
);
const ts = require("typescript");
const { createI18n } = require("vue-i18n");
const { parse, compileTemplate } = require("@vue/compiler-sfc");
const { NodeTypes } = require("@vue/compiler-dom");
const root = new URL("../frontend/src/", import.meta.url);
const catalogs = {};
for (const locale of ["es", "en", "pt"]) {
  const source = await readFile(new URL(`i18n/${locale}.ts`, root), "utf8");
  const code = ts.transpileModule(source, {
    compilerOptions: { module: ts.ModuleKind.ESNext },
  }).outputText;
  catalogs[locale] = (
    await import(
      `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`
    )
  ).messages;
}
const keys = Object.keys(catalogs.es).filter(
  (key) => key.startsWith("scope.") || key.startsWith("backendErrors."),
);
let checks = 0;
for (const locale of ["en", "pt"]) {
  assert.deepEqual(
    Object.keys(catalogs[locale]).sort(),
    Object.keys(catalogs.es).sort(),
    `${locale}: paridad de claves`,
  );
  for (const key of keys) {
    assert.equal(typeof catalogs[locale][key], "string", `${locale}: ${key}`);
    const params = (value) =>
      [...value.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();
    assert.deepEqual(
      params(catalogs[locale][key]),
      params(catalogs.es[key]),
      `${locale}: parámetros de ${key}`,
    );
    checks++;
  }
}
const warnings = [];
const i18n = createI18n({
  legacy: false,
  locale: "es",
  fallbackLocale: false,
  messages: catalogs,
  missing: (...args) => warnings.push(args),
});
for (const locale of ["es", "en", "pt"]) {
  i18n.global.locale.value = locale;
  for (const key of keys.filter((k) => k.startsWith("scope."))) {
    const params = Object.fromEntries(
      [...catalogs[locale][key].matchAll(/\{(\w+)\}/g)].map((m) => [
        m[1],
        "TEST",
      ]),
    );
    assert.equal(
      i18n.global.t(key, params),
      catalogs[locale][key].replace(/\{\w+\}/g, "TEST"),
      `${locale}: ${key}`,
    );
    checks++;
  }
}
assert.equal(warnings.length, 0, "Ninguna traducción usa fallback");
for (const filename of (await readdir(new URL("components/", root))).filter(
  (name) => /^(Scope.*|ProjectConversation|ConversationMessageEditor|ConversationNotifications|ConversationUnreadBadge)\.vue$/.test(name),
)) {
  const source = await readFile(
    new URL(`components/${filename}`, root),
    "utf8",
  );
  for (const match of source.matchAll(/\bt\(\s*(['"])([^'"]+)\1/g)) {
    for (const locale of ['es','en','pt']) {
      assert(Object.hasOwn(catalogs[locale],match[2]),`${filename}: clave ausente ${locale}: ${match[2]}`);
      checks++;
    }
  }
  const { descriptor } = parse(source);
  const compiled = compileTemplate({
    source: descriptor.template.content,
    filename,
    id: filename,
  });
  assert.equal(compiled.errors.length, 0, filename);
  const walk = (node) => {
    if (node.type === NodeTypes.TEXT)
      assert(
        !/[A-Za-zÁ-ú]{3}/.test(node.content.replace("SHA-256", "")),
        `${filename}: texto estático ${node.content}`,
      );
    for (const prop of node.props ?? []) {
      if (prop.type === NodeTypes.ATTRIBUTE)
        assert(
          !prop.name.includes(":"),
          `${filename}: atributo mal formado ${prop.name}`,
        );
      if (
        prop.type === NodeTypes.ATTRIBUTE &&
        ["label", "hint", "aria-label", "title", "placeholder"].includes(
          prop.name,
        )
      )
        assert.fail(`${filename}: atributo sin i18n ${prop.name}`);
    }
    for (const child of node.children ?? []) walk(child);
    for (const branch of node.branches ?? []) walk(branch);
  };
  walk(compiled.ast);
  checks++;
}
console.log(
  `${checks} comprobaciones de catálogos, interpolación y plantillas sin textos estáticos: OK`,
);

// Revisión opcional con el navegador de capture-ui.mjs y la cuenta de pruebas habitual.
export async function verifyScopeI18n({ evaluate, navigate, waitFor, output }) {
  const { writeFile } = await import("node:fs/promises");
  const checks = [];
  const api = async (path) =>
    evaluate(
      `fetch('/api/v1'+${JSON.stringify(path)},{credentials:'include'}).then(r=>r.json())`,
    );
  const query = encodeURIComponent(
    JSON.stringify({
      page: 1,
      limit: 50,
      order_by: null,
      order_asc: true,
      filters: [],
    }),
  );
  const projects = (await api(`/projects/search?q=${query}`)).items;
  const projectData = [];
  for (const project of projects)
    projectData.push({
      project,
      scope: await api(`/scope/projects/${project.id}`),
    });
  const accepted = projectData.find(
    (p) => p.scope.active && p.scope.revisions.length > 1,
  );
  const pending = projectData.find(
    (p) => !p.scope.active && p.scope.revisions.length > 0,
  );
  assert(
    accepted && pending,
    "Cuenta con alcance aceptado y propuesta pendiente",
  );
  const setLocale = async (locale) => {
    await evaluate(
      `document.querySelector('#q-app').__vue_app__.config.globalProperties.$i18n.locale=${JSON.stringify(locale)}`,
    );
  };
  const expect = async (expression, label) => {
    await waitFor(expression, label);
    checks.push(label);
  };
  await navigate(`/projects/${accepted.project.id}`);
  await waitFor(
    "!!document.querySelector('nav[aria-label]')",
    "documentos cargados",
  );
  for (const locale of ["en", "pt", "es"]) {
    await setLocale(locale);
    for (const key of [
      "scope.title",
      "scope.revisionDocuments",
      "scope.mainRole",
      "scope.downloadMarkdown",
      "scope.compare",
      "scope.conversation",
    ]) {
      const text = catalogs[locale][key];
      await expect(
        `document.querySelector('.scope-document').textContent.includes(${JSON.stringify(text)})`,
        `${locale}: ${key}`,
      );
    }
    await expect(
      `!!document.querySelector('nav[aria-label='+${JSON.stringify(JSON.stringify(catalogs[locale]["scope.documentIndex"]))}+']')`,
      `${locale}: índice accesible`,
    );
  }
  // Simula un error HTTP sin escribir en la BD; comprueba el resolver real del panel.
  for (const locale of ["en", "pt", "es"]) {
    await setLocale(locale);
    const message = "El alcance excede los límites de documentos/anexos.";
    await evaluate(`(() => {
      window.scopeOriginalFetch = window.fetch;
      window.fetch = (url, options) => String(url).includes('/scope/projects/')
        ? Promise.resolve(new Response(JSON.stringify({error:${JSON.stringify(message)},type:'bad_request'}),{status:400,headers:{'Content-Type':'application/json'}}))
        : window.scopeOriginalFetch(url, options);
      Array.from(document.querySelectorAll('.scope-document button')).find(b=>b.textContent.includes(${JSON.stringify(catalogs[locale]["scope.refresh"])})).click();
    })()`);
    await expect(
      `document.querySelector('.scope-document [role=alert]')?.textContent.includes(${JSON.stringify(catalogs[locale]["backendErrors." + message])})`,
      `${locale}: error HTTP traducido`,
    );
    await evaluate(
      "window.fetch = window.scopeOriginalFetch; delete window.scopeOriginalFetch",
    );
  }
  await navigate(`/projects/${pending.project.id}`);
  await waitFor(
    "!!document.querySelector('.scope-document')",
    "propuesta cargada",
  );
  await evaluate(
    `Array.from(document.querySelectorAll('.scope-document button')).find(b=>b.textContent.includes(${JSON.stringify(catalogs.es["scope.edit"])})).click()`,
  );
  await waitFor(
    "!!document.querySelector('.scope-document input[type=file]')",
    "editor abierto",
  );
  for (const locale of ["en", "pt", "es"]) {
    await setLocale(locale);
    for (const key of [
      "scope.documentsTitle",
      "scope.addDocument",
      "scope.importMarkdown",
      "scope.publicationSnapshot",
    ]) {
      await expect(
        `document.querySelector('.scope-document').textContent.includes(${JSON.stringify(catalogs[locale][key])})`,
        `${locale}: editor ${key}`,
      );
    }
    const role = catalogs[locale]["scope.mainOption"];
    await expect(
      `document.querySelector('.scope-document').textContent.includes(${JSON.stringify(role)})`,
      `${locale}: opciones reactivas`,
    );
  }
  await writeFile(
    `${output}/scope-i18n.json`,
    JSON.stringify({ checks }, null, 2),
  );
  console.log(`${checks.length} comprobaciones i18n en navegador: OK`);
}
