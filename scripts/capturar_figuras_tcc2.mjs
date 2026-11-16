#!/usr/bin/env node
// Captura as telas usadas como figuras no Relatório de TCC II.
// Usa o Chrome local em modo headless por um pipe DevTools isolado (mesmo
// mecanismo de scripts/capture-ui.mjs): nenhuma janela é aberta e o perfil
// pessoal do navegador não é usado. Não altera dados: apenas navega, abre um
// diálogo e o fecha sem salvar.
//
// Pré-requisitos: base horbyta_development com o seed, backend em execução e o
// build do frontend servido (ex.: node scripts/serve-ui-build.mjs frontend/dist/pwa 9100).
//
// Uso: node scripts/capturar_figuras_tcc2.mjs [URL] [DIRETÓRIO]
//   URL padrão: http://127.0.0.1:9100
//   Diretório padrão: docs/screenshots/tcc2
import { spawn } from "node:child_process";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { StringDecoder } from "node:string_decoder";

const base = new URL(process.argv[2] || "http://127.0.0.1:9100");
const output = resolve(process.argv[3] || "docs/screenshots/tcc2");
const LOCALE = "pt";

// Contas e registros fictícios criados pelo seed de desenvolvimento (README).
const FREELANCER = { email: "admin@horbyta.test", password: "Admin@123" };
const CLIENTE = { email: "cliente3@horbyta.test", password: "Cliente@123" };
const IDS = {
  admin: "01a1565e-c0f4-74e0-800a-34bc25af808a",
  projetoComMudancaAprovada: "01a1565e-c3e0-7621-9578-9c4172e5b67e", // Panel de Atlas
  mudancaAprovada: "01a1565e-c46b-7781-9480-a4bfc048a090",
  mudancaEnviada: "01a1565e-c462-7572-8f72-cb1066f9a87b",
  mudancaAvaliada: "01a1565e-c465-7d71-8d2b-9909dfb8ac92",
};

await mkdir(output, { recursive: true });
const profile = await mkdtemp(join(tmpdir(), "horbyta-tcc2-"));
const chrome = spawn(
  process.env.CHROME_BIN || "google-chrome",
  [
    "--headless",
    "--remote-debugging-pipe",
    `--user-data-dir=${profile}`,
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-dev-shm-usage",
    "--disable-extensions",
    "--disable-gpu",
    "--hide-scrollbars",
    "--force-prefers-reduced-motion",
    "--lang=pt-BR",
  ],
  { stdio: ["ignore", "ignore", "pipe", "pipe", "pipe"] },
);

let nextId = 0;
let buffer = "";
let stderr = "";
const pending = new Map();
const decoder = new StringDecoder("utf8");
chrome.stderr.on("data", (c) => (stderr = (stderr + c).slice(-4000)));
chrome.on("exit", (code) => {
  for (const { reject } of pending.values()) reject(new Error(`Chrome saiu (${code}). ${stderr}`));
});
chrome.stdio[4].on("data", (chunk) => {
  buffer += decoder.write(chunk);
  let end;
  while ((end = buffer.indexOf("\0")) !== -1) {
    const raw = buffer.slice(0, end);
    buffer = buffer.slice(end + 1);
    if (!raw) continue;
    const msg = JSON.parse(raw);
    const entry = pending.get(msg.id);
    if (!entry) continue;
    pending.delete(msg.id);
    if (msg.error) entry.reject(new Error(msg.error.message));
    else entry.resolve(msg.result);
  }
});
function send(method, params = {}, sessionId) {
  const id = ++nextId;
  return new Promise((res, rej) => {
    pending.set(id, { resolve: res, reject: rej });
    chrome.stdio[3].write(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }) + "\0");
  });
}

const capturas = [];
try {
  const { targetId } = await send("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await send("Target.attachToTarget", { targetId, flatten: true });
  const call = (m, p) => send(m, p, sessionId);
  await call("Page.enable");
  await call("Runtime.enable");
  await call("Emulation.setDeviceMetricsOverride", { width: 1440, height: 900, deviceScaleFactor: 2, mobile: false });
  await call("Emulation.setEmulatedMedia", { features: [{ name: "prefers-color-scheme", value: "light" }] });

  const evaluate = async (expression) => {
    const r = await call("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || r.exceptionDetails.text);
    return r.result.value;
  };
  const waitFor = async (condition, label, ms = 20000) => {
    const deadline = Date.now() + ms;
    while (Date.now() < deadline) {
      if (await evaluate(condition).catch(() => false)) return;
      await delay(150);
    }
    throw new Error(`Tempo esgotado aguardando: ${label}`);
  };
  const semCarregamento =
    "!Array.from(document.querySelectorAll('.q-page .q-spinner, .q-inner-loading, .q-loading, .q-skeleton')).some(el => el.checkVisibility({ opacityProperty: true, visibilityProperty: true }))";
  const estabilizar = async () => {
    await waitFor(semCarregamento, "carregamento da página");
    await evaluate("document.fonts.ready.then(() => true)");
    await delay(1200); // animações e gráficos
  };
  // O idioma do portal fica em memória; navegar pelo roteador do Vue o preserva.
  const definirIdioma = () =>
    evaluate(`(() => { const i18n = document.querySelector('#q-app').__vue_app__.config.globalProperties.$i18n;
      if (typeof i18n.locale === 'object') i18n.locale.value = '${LOCALE}'; else i18n.locale = '${LOCALE}'; return true; })()`);
  const irPara = async (path) => {
    await evaluate(`document.querySelector('#q-app').__vue_app__.config.globalProperties.$router.push(${JSON.stringify(path)}).then(() => true)`);
    await waitFor(`location.pathname === ${JSON.stringify(path)} && !!document.querySelector('.q-page')`, path);
    await definirIdioma();
    await estabilizar();
  };
  const capturar = async (arquivo, descricao) => {
    await evaluate("window.scrollTo(0, 0), document.activeElement?.blur?.(), true");
    const { data } = await call("Page.captureScreenshot", { format: "png" });
    await writeFile(join(output, arquivo), Buffer.from(data, "base64"));
    capturas.push({ arquivo, descricao });
    console.log(`Capturada: ${arquivo}`);
  };
  // Telas de detalhe: janela mais alta para incluir os impactos da oferta.
  const capturarAlto = async (arquivo, descricao, altura = 1750) => {
    await call("Emulation.setDeviceMetricsOverride", { width: 1440, height: altura, deviceScaleFactor: 2, mobile: false });
    await delay(800);
    await capturar(arquivo, descricao);
    await call("Emulation.setDeviceMetricsOverride", { width: 1440, height: 900, deviceScaleFactor: 2, mobile: false });
  };
  const login = async (path, conta, destino) => {
    await call("Page.navigate", { url: new URL(path, base).href });
    await waitFor("!!document.querySelector('input[type=email]')", `formulário de login ${path}`);
    await definirIdioma();
    await evaluate(`(() => {
      const v = ${JSON.stringify(conta)};
      for (const t of ['email', 'password']) {
        const i = document.querySelector('input[type=' + t + ']');
        i.value = v[t];
        i.dispatchEvent(new Event('input', { bubbles: true }));
      }
      document.querySelector('form').requestSubmit();
      return true;
    })()`);
    await waitFor(`location.pathname === ${JSON.stringify(destino)} && !!document.querySelector('.q-header')`, `login em ${path}`);
    await definirIdioma();
    await estabilizar();
  };

  // ---- Área do freelancer ----
  await login("/login", FREELANCER, "/");
  await capturar("01-freelancer-dashboard.png", "Painel do freelancer");

  await irPara(`/projects/${IDS.projetoComMudancaAprovada}`);
  await capturar("02-freelancer-projeto-escopo.png", "Detalhe do projeto com escopo aceito");

  await irPara("/change-requests");
  await capturar("03-freelancer-solicitacoes.png", "Listagem de solicitações de mudança");

  await irPara(`/change-requests/${IDS.mudancaEnviada}`);
  await evaluate(`(() => {
    const b = Array.from(document.querySelectorAll('.q-btn')).find(el => el.querySelector('.q-icon')?.textContent.trim() === 'analytics' && el.checkVisibility());
    if (!b) throw new Error('Botão Avaliar não encontrado');
    b.click(); return true; })()`);
  await waitFor("!!document.querySelector('.q-dialog .q-card')", "diálogo de avaliação");
  await delay(800);
  await capturar("04-freelancer-avaliacao-impacto.png", "Diálogo de avaliação de impacto");
  await evaluate("document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', keyCode: 27, bubbles: true })), true");
  await delay(500);

  await irPara(`/change-requests/${IDS.mudancaAprovada}`);
  await capturarAlto("05-freelancer-solicitacao-aprovada.png", "Detalhe de solicitação aprovada", 1560);

  // ---- Portal do cliente (sessão separada) ----
  await call("Network.enable");
  await call("Network.clearBrowserCookies");
  await login(`/client/u/${IDS.admin}/login`, CLIENTE, "/client");
  await capturar("06-cliente-dashboard.png", "Painel do cliente");

  await irPara(`/client/change-requests/${IDS.mudancaAvaliada}`);
  await capturarAlto("07-cliente-decisao-mudanca.png", "Solicitação avaliada no portal do cliente", 1560);

  await writeFile(join(output, "capturas.json"), JSON.stringify(capturas, null, 2));
} catch (error) {
  console.error(error);
  process.exitCode = 1;
} finally {
  const saiu = new Promise((r) => chrome.once("exit", r));
  chrome.kill();
  await Promise.race([saiu, delay(5000)]);
  await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 }).catch(() => {});
}
