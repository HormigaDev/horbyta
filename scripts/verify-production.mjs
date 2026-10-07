import { writeFile } from "node:fs/promises";
import { join } from "node:path";

// Build-only checks: no dev-store imports, isolated profile, actual API authentication.
export async function verifyProduction({
  call,
  evaluate,
  navigate,
  waitFor,
  output,
  account,
  loginAssets,
}) {
  const checks = [];
  const check = async (label, expression) => {
    if (!(await evaluate(expression)))
      throw Error("Production check failed: " + label);
    checks.push(label);
    console.log("Verified production: " + label);
  };
  await check(
    "Production assets, without Vite development modules",
    "performance.getEntriesByType('resource').some(e=>new URL(e.name).pathname.startsWith('/assets/index-')) && !performance.getEntriesByType('resource').some(e=>e.name.includes('/@vite/client')||e.name.includes('/src/'))",
  );
  await check(
    "Build authenticates actual dedicated account through API proxy",
    "fetch('/api/v1/users/me').then(r=>r.json()).then(u=>u.email==='ui-tests@horbyta.test')",
  );
  const manifest = await evaluate(
    "fetch(document.querySelector('link[rel=manifest]').href).then(r=>r.json())",
  );
  if (
    !manifest.theme_color ||
    !manifest.background_color ||
    !manifest.icons.length
  )
    throw Error("Incomplete PWA manifest");
  checks.push("PWA manifest and icons available");
  await waitFor(
    "navigator.serviceWorker.controller!==null",
    "production service worker controller",
  );
  checks.push("Production service worker controls the page");
  await navigate("/tasks");
  for (const width of [320, 360, 768, 1440]) {
    await call("Emulation.setDeviceMetricsOverride", {
      width,
      height: 900,
      deviceScaleFactor: 1,
      mobile: width < 600,
    });
    await check(
      "Built task table reflows at " + width,
      "document.documentElement.scrollWidth<=innerWidth+1 && !!document.querySelector('.q-table')",
    );
  }
  await evaluate(
    `document.querySelector('button[aria-label="Configurar columnas"]').focus()`,
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
    "built column dialog focus",
  );
  await evaluate("new Promise(r=>setTimeout(r,300))");
  const { nodes } = await call("Accessibility.getFullAXTree");
  const nameless = nodes.filter(
    (n) =>
      !n.ignored &&
      ["button", "checkbox", "combobox"].includes(n.role?.value) &&
      !n.name?.value?.trim() &&
      !n.properties?.some((p) => p.name === "disabled" && p.value.value),
  );
  if (nameless.length)
    throw Error("Unnamed built modal controls: " + nameless.length);
  checks.push("Built column dialog controls have accessible names");
  for (const modifiers of [0, 8]) {
    for (let step = 0; step < 40; step++) {
      for (const type of ["keyDown", "keyUp"])
        await call("Input.dispatchKeyEvent", {
          type,
          key: "Tab",
          code: "Tab",
          windowsVirtualKeyCode: 9,
          modifiers,
        });
      await evaluate("new Promise(r=>setTimeout(r,50))");
      const focus = await evaluate(
        "({inside:!!document.activeElement.closest('.q-dialog'),hasFocus:document.hasFocus(),html:document.activeElement.outerHTML.slice(0,300)})",
      );
      if (!focus.inside) {
        await writeFile(
          join(output, "production-focus-failure.json"),
          JSON.stringify({ modifiers, step, focus }, null, 2),
        );
        throw Error("Built modal focus escaped");
      }
    }
    checks.push(
      "Built modal contains 40 " +
        (modifiers ? "Shift+Tab" : "Tab") +
        " steps including wrap",
    );
  }
  for (const type of ["keyDown", "keyUp"])
    await call("Input.dispatchKeyEvent", {
      type,
      key: "Escape",
      code: "Escape",
      windowsVirtualKeyCode: 27,
    });
  await waitFor("!document.querySelector('.q-dialog')", "built modal close");
  await check(
    "Built modal returns focus to trigger",
    "document.activeElement.getAttribute('aria-label')==='Configurar columnas'",
  );
  await call("Network.clearBrowserCache");
  await call("Network.setCacheDisabled", { cacheDisabled: true });
  await call("Network.setBlockedURLs", {
    urls: ["*fonts.googleapis.com*", "*fonts.gstatic.com*"],
  });
  await navigate("/settings");
  await check(
    "Built application keeps locally served fonts",
    "Array.from(document.fonts).some(f=>f.status==='loaded') && !!document.querySelector('.q-page')",
  );
  await call("Network.setBlockedURLs", { urls: [] });
  await call("Network.setCacheDisabled", { cacheDisabled: false });
  const blocked = await call("Page.addScriptToEvaluateOnNewDocument", {
    source:
      "for(const name of ['getItem','setItem'])Storage.prototype[name]=function(){throw new DOMException('Blocked browser storage','SecurityError')};",
  });
  try {
    await navigate("/tasks");
    await check(
      "Built app loads authenticated table with browser storage blocked",
      "!!document.querySelector('.q-table') && !!document.querySelector('.q-header')",
    );
  } finally {
    await call("Page.removeScriptToEvaluateOnNewDocument", {
      identifier: blocked.identifier,
    });
  }
  await navigate("/settings");
  const fonts = await evaluate(
    "performance.getEntriesByType('resource').filter(e=>/woff2?(\\?|$)/.test(e.name)).map(e=>new URL(e.name).pathname)",
  );
  await evaluate("fetch('/api/v1/auth/logout',{method:'POST'})");
  await navigate("/login");
  for (const width of [320, 360, 768, 1440])
    for (const language of ["Español", "English", "Português"]) {
      await call("Emulation.setDeviceMetricsOverride", {
        width,
        height: 900,
        deviceScaleFactor: 1,
        mobile: width < 600,
      });
      await evaluate("document.querySelector('.auth-tools .q-select').click()");
      await waitFor(
        "!!document.querySelector('.q-menu .q-item')",
        "public language menu",
      );
      await evaluate(
        `Array.from(document.querySelectorAll('.q-menu .q-item')).find(e=>e.textContent.includes(${JSON.stringify(language)})).click()`,
      );
      await waitFor(
        "!document.querySelector('.q-menu')",
        "public language applied",
      );
      await evaluate(
        "new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)))",
      );
      await check(
        "Built anonymous login " + language + " " + width,
        "document.documentElement.scrollWidth<=innerWidth+1 && !!document.querySelector('input[type=email][autocomplete=username]')",
      );
      if (width === 320) {
        const { data } = await call("Page.captureScreenshot", {
          format: "png",
          captureBeyondViewport: false,
        });
        await writeFile(
          join(output, "production-login-320-" + language + ".png"),
          Buffer.from(data, "base64"),
        );
      }
    }
  await evaluate("document.querySelector('.auth-tools .q-select').click()");
  await waitFor(
    "!!document.querySelector('.q-menu .q-item')",
    "restore public language",
  );
  await evaluate(
    "Array.from(document.querySelectorAll('.q-menu .q-item')).find(e=>e.textContent.includes('Español')).click()",
  );
  await call("Network.emulateNetworkConditions", {
    offline: true,
    latency: 0,
    downloadThroughput: -1,
    uploadThroughput: -1,
  });
  try {
    await navigate("/login");
    await check(
      "Cached PWA shell opens login while offline",
      "!!document.querySelector('input[type=email]') && !!document.querySelector('input[type=password]') && document.documentElement.scrollWidth<=innerWidth+1",
    );
    const { data } = await call("Page.captureScreenshot", {
      format: "png",
      captureBeyondViewport: false,
    });
    await writeFile(
      join(output, "production-offline-login.png"),
      Buffer.from(data, "base64"),
    );
  } finally {
    await call("Network.emulateNetworkConditions", {
      offline: false,
      latency: 0,
      downloadThroughput: -1,
      uploadThroughput: -1,
    });
  }
  await navigate("/login");
  await evaluate(
    `(()=>{const values=${JSON.stringify({ email: account.email, password: account.password })};for(const type of ['email','password']){const e=document.querySelector('input[type='+type+']');e.value=values[type];e.dispatchEvent(new Event('input',{bubbles:true}));}document.querySelector('form').requestSubmit();})()`,
  );
  await waitFor(
    "location.pathname==='/' && !!document.querySelector('.q-header')",
    "restore production login",
  );
  await writeFile(
    join(output, "production-verification.json"),
    JSON.stringify(
      {
        checkedAt: new Date().toISOString(),
        checks,
        loginAssets,
        fonts,
        manifest,
        scope:
          "Local production build and test proxy, isolated Chrome profile. Offline shell only; business data still require API. Service worker precaching is separate from lazy script execution.",
      },
      null,
      2,
    ),
  );
  console.log("Production checks passed: " + checks.length);
}
