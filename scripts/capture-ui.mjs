#!/usr/bin/env node
// Headless Chrome over an isolated DevTools pipe: no visible window or personal profile.
import { spawn } from "node:child_process";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { pathToFileURL } from "node:url";
import { StringDecoder } from "node:string_decoder";

if (process.argv.includes("--help")) {
  console.log(`Usage: node scripts/capture-ui.mjs [URL] [ROUTE ...]
Default URL: http://127.0.0.1:9000
Default routes: / /tasks /projects /board /tasks/new /settings
Credentials: UI_TEST_EMAIL + UI_TEST_PASSWORD, or target/ui-tests/account.json
Output: target/ui-tests/screenshots (override UI_SCREENSHOT_DIR)
Chrome: google-chrome (override CHROME_BIN)
Set CHROME_NO_SANDBOX=1 only if the execution environment cannot run Chrome's sandbox.`);
  process.exit(0);
}

const base = new URL(process.argv[2] || "http://127.0.0.1:9000");
const routes =
  process.argv.length > 3
    ? process.argv.slice(3)
    : ["/", "/tasks", "/projects", "/board", "/tasks/new", "/settings"];
const output = resolve(
  process.env.UI_SCREENSHOT_DIR || "target/ui-tests/screenshots",
);
const account =
  process.env.UI_TEST_EMAIL && process.env.UI_TEST_PASSWORD
    ? {
        email: process.env.UI_TEST_EMAIL,
        password: process.env.UI_TEST_PASSWORD,
      }
    : JSON.parse(
        await readFile(resolve("target/ui-tests/account.json"), "utf8"),
      );
if (!account.email || !account.password)
  throw new Error("Test account email and password are required.");
await mkdir(output, { recursive: true });
const profile = await mkdtemp(join(tmpdir(), "horbyta-headless-"));
const args = [
  "--headless",
  "--remote-debugging-pipe",
  `--user-data-dir=${profile}`,
  "--no-first-run",
  "--no-default-browser-check",
  "--disable-dev-shm-usage",
  "--disable-background-networking",
  "--disable-extensions",
];
if (process.env.CHROME_NO_SANDBOX === "1") args.push("--no-sandbox");
const chrome = spawn(process.env.CHROME_BIN || "google-chrome", args, {
  stdio: ["ignore", "ignore", "pipe", "pipe", "pipe"],
});
let nextId = 0;
let buffer = "";
let stderr = "";
let failed = false;
const pending = new Map();
const diagnostics = [];
const inflight = new Map();
const decoder = new StringDecoder("utf8");
let lastActivity = Date.now();
let navigationLoader = null;

function rejectPending(error) {
  failed = true;
  for (const { reject, timer } of pending.values()) {
    clearTimeout(timer);
    reject(error);
  }
  pending.clear();
}
chrome.on("error", (error) => rejectPending(error));
chrome.on("exit", (code) =>
  rejectPending(new Error(`Chrome exited (${code}). ${stderr.slice(-1500)}`)),
);
chrome.stderr.on("data", (chunk) => {
  stderr = (stderr + chunk).slice(-4000);
});
chrome.stdio[3].on("error", (error) => rejectPending(error));
chrome.stdio[4].on("data", (chunk) => {
  buffer += decoder.write(chunk);
  let end;
  while ((end = buffer.indexOf("\0")) !== -1) {
    const raw = buffer.slice(0, end);
    buffer = buffer.slice(end + 1);
    if (!raw) continue;
    const message = JSON.parse(raw);
    const entry = pending.get(message.id);
    if (entry) {
      pending.delete(message.id);
      clearTimeout(entry.timer);
      if (message.error) entry.reject(new Error(message.error.message));
      else entry.resolve(message.result);
    } else if (message.method === "Network.requestWillBeSent") {
      const { requestId, request, type, loaderId } = message.params;
      if (
        request.url.startsWith(base.origin + "/") &&
        ["Fetch", "XHR"].includes(type) &&
        loaderId === navigationLoader
      ) {
        inflight.set(requestId, request.url);
        lastActivity = Date.now();
      }
    } else if (
      ["Network.loadingFinished", "Network.loadingFailed"].includes(
        message.method,
      )
    ) {
      if (inflight.delete(message.params.requestId)) lastActivity = Date.now();
    } else if (message.method === "Runtime.exceptionThrown") {
      diagnostics.push({
        type: "exception",
        text: message.params.exceptionDetails.text,
      });
    } else if (
      message.method === "Network.responseReceived" &&
      message.params.response.status >= 400
    ) {
      const response = message.params.response;
      diagnostics.push({
        type: "http",
        status: response.status,
        url: response.url,
      });
    }
  }
});

function send(method, params = {}, sessionId) {
  if (failed) return Promise.reject(new Error("Chrome is unavailable."));
  const id = ++nextId;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error(`Timed out: ${method}`));
    }, 30000);
    pending.set(id, { resolve, reject, timer });
    chrome.stdio[3].write(
      JSON.stringify({
        id,
        method,
        params,
        ...(sessionId ? { sessionId } : {}),
      }) + "\0",
    );
  });
}

try {
  const { targetId } = await send("Target.createTarget", {
    url: "about:blank",
  });
  const { sessionId } = await send("Target.attachToTarget", {
    targetId,
    flatten: true,
  });
  const call = (method, params = {}) => send(method, params, sessionId);
  await call("Page.enable");
  await call("Runtime.enable");
  await call("Network.enable");
  if (process.env.UI_VERIFY_CLIENT_STATES === "1") {
    const { installClientFixtures } = await import(
      pathToFileURL(resolve("scripts/verify-client-states.mjs")).href
    );
    await installClientFixtures(call);
  }

  const evaluate = async (expression) => {
    const value = await call("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (value.exceptionDetails) throw new Error(value.exceptionDetails.text);
    return value.result.value;
  };
  const waitFor = async (condition, label) => {
    const deadline = Date.now() + 20000;
    while (Date.now() < deadline) {
      if (await evaluate(condition)) return;
      await delay(150);
    }
    throw new Error(
      `Timed out waiting for ${label}. Check the app and test credentials.`,
    );
  };
  const navigate = async (path) => {
    const target = new URL(path, base);
    if (target.origin !== base.origin)
      throw new Error("Capture routes must stay on the app origin.");
    // A document navigation cancels requests from the previous document.
    inflight.clear();
    lastActivity = Date.now();
    const response = await call("Page.navigate", { url: target.href });
    if (response.errorText) throw new Error(response.errorText);
    navigationLoader = response.loaderId ?? navigationLoader;
    inflight.clear();
    lastActivity = Date.now();
    await waitFor(
      `location.pathname === ${JSON.stringify(target.pathname)} && !!document.querySelector('.q-page, .not-found-page')`,
      path,
    );
    await evaluate("document.fonts.ready.then(() => true)");
    const deadline = Date.now() + 25000;
    let settled = false;
    while (Date.now() < deadline) {
      if (
        inflight.size === 0 &&
        Date.now() - lastActivity >= 600 &&
        (await evaluate(
          "!Array.from(document.querySelectorAll('.q-page .q-spinner, .q-inner-loading, .q-loading')).some(el => el.checkVisibility({ opacityProperty: true, visibilityProperty: true }))",
        ))
      ) {
        settled = true;
        break;
      }
      await delay(150);
    }
    if (!settled) {
      const state = await evaluate(
        "({ path: location.pathname, ready: document.readyState, spinners: Array.from(document.querySelectorAll('.q-page .q-spinner')).map(el => ({ visible: el.checkVisibility({ opacityProperty: true, visibilityProperty: true }), classes: el.getAttribute('class') })) })",
      );
      throw new Error(
        `Page did not settle: ${path}. ${JSON.stringify({ pending: [...inflight.values()], state })}`,
      );
    }
  };

  await call("Emulation.setDeviceMetricsOverride", {
    width: 1440,
    height: 1000,
    deviceScaleFactor: 1,
    mobile: false,
  });
  await navigate("/login");
  const loginAssets = await evaluate(
    "performance.getEntriesByType('resource').filter(e=>['script','link','css'].includes(e.initiatorType)).map(e=>({path:new URL(e.name).pathname,bytes:e.transferSize,duration:Math.round(e.duration)}))",
  );
  if (process.env.UI_VERIFY_CLIENT_STATES !== "1") {
    await waitFor(
      "!!document.querySelector('input[type=email]')",
      "login form",
    );
    await evaluate(`(() => {
    const values = ${JSON.stringify({ email: account.email, password: account.password })};
    for (const type of ['email', 'password']) {
      const input = document.querySelector('input[type=' + type + ']');
      input.value = values[type];
      input.dispatchEvent(new Event('input', { bubbles: true }));
      input.dispatchEvent(new Event('change', { bubbles: true }));
    }
    document.querySelector('form').requestSubmit();
  })()`);
    await waitFor(
      "location.pathname === '/' && !!document.querySelector('.q-header')",
      "authenticated dashboard",
    );
    const authenticated = await evaluate(
      "fetch('/api/v1/health', { credentials: 'include' }).then(response => response.ok)",
    );
    if (!authenticated)
      throw new Error("The test account did not authenticate.");
  } else {
    const { verifyClientStates } = await import(
      pathToFileURL(resolve("scripts/verify-client-states.mjs")).href
    );
    await verifyClientStates({ call, evaluate, navigate, waitFor, output });
  }

  if (process.env.UI_VERIFY === "1") {
    const { verify } = await import(
      pathToFileURL(resolve("scripts/verify-ui.mjs")).href
    );
    await verify({ call, evaluate, navigate, waitFor, output });
  }
  if (process.env.UI_VERIFY_FORMS === "1") {
    const { verifyForms } = await import(
      pathToFileURL(resolve("scripts/verify-forms.mjs")).href
    );
    await verifyForms({ call, evaluate, navigate, waitFor, output });
  }
  if (process.env.UI_VERIFY_WORKFLOWS === "1") {
    const { verifyWorkflows } = await import(
      pathToFileURL(resolve("scripts/verify-workflows.mjs")).href
    );
    await verifyWorkflows({ call, evaluate, navigate, waitFor, output });
  }
  if (process.env.UI_VERIFY_DASHBOARD_PORTAL === "1") {
    const { verifyDashboardPortal } = await import(
      pathToFileURL(resolve("scripts/verify-dashboard-portal.mjs")).href
    );
    await verifyDashboardPortal({
      call,
      evaluate,
      navigate,
      waitFor,
      output,
      account,
    });
  }
  if (process.env.UI_VERIFY_MIGRATION === "1") {
    const { verifyMigration } = await import(
      pathToFileURL(resolve("scripts/verify-migration.mjs")).href
    );
    await verifyMigration({
      call,
      evaluate,
      navigate,
      waitFor,
      output,
      account,
    });
  }
  if (process.env.UI_VERIFY_PRODUCTION === "1") {
    const { verifyProduction } = await import(
      pathToFileURL(resolve("scripts/verify-production.mjs")).href
    );
    await verifyProduction({
      call,
      evaluate,
      navigate,
      waitFor,
      output,
      account,
      loginAssets,
    });
  }
  const captures = [];
  for (const viewport of [
    { name: "desktop", width: 1440, height: 1000, mobile: false },
    { name: "mobile", width: 360, height: 800, mobile: true },
  ]) {
    await call("Emulation.setDeviceMetricsOverride", {
      width: viewport.width,
      height: viewport.height,
      deviceScaleFactor: 1,
      mobile: viewport.mobile,
    });
    await call("Emulation.setTouchEmulationEnabled", {
      enabled: viewport.mobile,
    });
    for (const route of routes) {
      await navigate(route);
      const state = await evaluate(`({
        route: location.pathname,
        theme: document.body.dataset.theme || document.documentElement.dataset.theme,
        pageWidth: document.documentElement.scrollWidth,
        viewportWidth: innerWidth,
        pageHeight: document.documentElement.scrollHeight
      })`);
      const { data } = await call("Page.captureScreenshot", {
        format: "png",
        captureBeyondViewport: false,
      });
      const file = `${viewport.name}-${route.replace(/[^a-z0-9]+/gi, "-").replace(/^-|-$/g, "") || "dashboard"}.png`;
      await writeFile(join(output, file), Buffer.from(data, "base64"));
      captures.push({ file, ...state, viewport });
      console.log(`Captured ${viewport.name}: ${route}`);
    }
  }
  await writeFile(
    join(output, "report.json"),
    JSON.stringify(
      {
        capturedAt: new Date().toISOString(),
        loginAssets,
        captures,
        diagnostics,
      },
      null,
      2,
    ),
  );
  console.log(`Screenshots and report: ${output}`);
} finally {
  if (!failed) {
    try {
      await send("Browser.close");
    } catch {
      /* Chrome can exit before acknowledging close. */
    }
  }
  if (chrome.exitCode === null) {
    const closed = new Promise((resolve) => chrome.once("exit", resolve));
    chrome.kill("SIGTERM");
    await Promise.race([closed, delay(3000)]);
    if (chrome.exitCode === null) {
      chrome.kill("SIGKILL");
      await closed;
    }
  }
  await rm(profile, {
    recursive: true,
    force: true,
    maxRetries: 10,
    retryDelay: 100,
  });
}
