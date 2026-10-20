// Cada código de src/error_codes.rs necesita su traducción backendErrors.<código> en es/en/pt,
// con los mismos parámetros $n, y vue-i18n debe devolver el texto sin interpretarlo.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";

const require = createRequire(new URL("../frontend/package.json", import.meta.url));
const ts = require("typescript");
const { createI18n } = require("vue-i18n");
const load = async (path) => {
  const source = await readFile(new URL(path, import.meta.url), "utf8");
  const code = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ESNext } }).outputText;
  return import(`data:text/javascript;base64,${Buffer.from(code).toString("base64")}`);
};

const rust = await readFile(new URL("../src/error_codes.rs", import.meta.url), "utf8");
const codes = new Map(
  [...rust.matchAll(/^\s+([A-Z0-9_]+) => "((?:[^"\\]|\\.)*)",$/gm)].map((m) => [m[1], m[2]]),
);
assert.ok(codes.size > 300, "catálogo de errores leído");
const params = (text) => [...new Set(text.match(/\$\d+/g) ?? [])].sort().join(",");

const catalogs = {};
for (const locale of ["es", "en", "pt"]) {
  catalogs[locale] = (await load(`../frontend/src/i18n/${locale}.ts`)).messages;
}
const i18n = createI18n({ legacy: false, locale: "es", fallbackLocale: false, messages: catalogs, missingWarn: false });
const { translateApiError } = await load("../frontend/src/utils/apiErrors.ts");
let checks = 0;
for (const locale of ["es", "en", "pt"]) {
  const messages = catalogs[locale];
  i18n.global.locale.value = locale;
  const translator = {
    t: (key) => i18n.global.t(key),
    has: (key) => Object.prototype.hasOwnProperty.call(messages, key),
  };
  for (const [code, message] of codes) {
    const key = `backendErrors.${code}`;
    assert.equal(typeof messages[key], "string", `${locale}: falta ${key}`);
    assert.equal(params(messages[key]), params(message), `${locale}: parámetros de ${key}`);
    assert.equal(i18n.global.t(key), messages[key], `${locale}: vue-i18n altera ${key}`);
    const sample = (message.match(/\$\d+/g) ?? []).map((_, index) => `P${index + 1}`);
    assert.ok(!translateApiError(translator, code, sample).includes("$"), `${locale}: ${key} sin sustituir`);
    checks += 4;
  }
  if (locale === "pt") {
    const untranslated = [...codes.keys()].filter(
      (code) => messages[`backendErrors.${code}`] === catalogs.en[`backendErrors.${code}`],
    );
    assert.deepEqual(untranslated, [], "pt: errores sin traducir (iguales al inglés)");
  }
  const extra = Object.keys(messages).filter(
    (key) => key.startsWith("backendErrors.") && !codes.has(key.slice("backendErrors.".length)),
  );
  assert.deepEqual(extra, [], `${locale}: traducciones sin código en el backend`);
}
console.log(`${codes.size} códigos de error, ${checks} comprobaciones: OK`);
