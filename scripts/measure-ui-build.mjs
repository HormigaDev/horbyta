#!/usr/bin/env node
import { readFile, readdir, stat } from "node:fs/promises";
import { resolve, join, dirname } from "node:path";
import { gzipSync } from "node:zlib";
const root = resolve(process.argv[2] || "frontend/dist/pwa");
const files = [];
async function walk(path) {
  for (const entry of await readdir(path, { withFileTypes: true })) {
    const file = join(path, entry.name);
    if (entry.isDirectory()) await walk(file);
    else files.push(file);
  }
}
await walk(root);
const html = await readFile(join(root, "index.html"), "utf8");
const entries = [...html.matchAll(/(?:src|href)="([^"?#]+\.(?:js|css))"/g)].map(
  (m) => resolve(root, "." + (m[1].startsWith("/") ? m[1] : "/" + m[1])),
);
const initial = new Set();
async function imports(file, seen = initial) {
  if (seen.has(file)) return;
  seen.add(file);
  if (!file.endsWith(".js")) return;
  const source = await readFile(file, "utf8");
  for (const m of source.matchAll(
    /(?:\bfrom\s*|\bimport\s*)["'](\.\.?\/[^"']+\.js)["']/g,
  ))
    await imports(resolve(dirname(file), m[1]), seen);
}
for (const entry of entries) await imports(entry);
async function metrics(list) {
  let bytes = 0,
    gzip = 0;
  for (const file of list) {
    const content = await readFile(file);
    bytes += content.length;
    gzip += gzipSync(content).length;
  }
  return { files: list.length, bytes, gzip };
}
const result = {
  directory: root,
  initialStaticGraph: await metrics([...initial]),
  initialFiles: [...initial].map((f) => f.slice(root.length + 1)),
  javascript: await metrics(files.filter((f) => f.endsWith(".js"))),
  css: await metrics(files.filter((f) => f.endsWith(".css"))),
  fonts: await metrics(files.filter((f) => /\.woff2?$/.test(f))),
  totalBytes: (await Promise.all(files.map((f) => stat(f)))).reduce(
    (n, s) => n + s.size,
    0,
  ),
  method:
    "HTML script/style entries and recursive static JS imports, excluding dynamic imports. Sum of individual gzip sizes, not a network waterfall. PWA precache may download lazy modules in background.",
};
if (process.argv[3]) {
  const report = JSON.parse(await readFile(resolve(process.argv[3]), "utf8"));
  const login = new Set(initial);
  for (const entry of report.loginAssets ?? [])
    if (entry.path.startsWith("/assets/") && /\.(js|css)$/.test(entry.path))
      await imports(resolve(root, "." + entry.path), login);
  result.observedLoginGraph = await metrics([...login]);
  result.loginFiles = [...login].map((f) => f.slice(root.length + 1));
  result.method +=
    " Observed cold login includes captured script/style imports, startup boot and their static dependencies; external font services are excluded.";
}
console.log(JSON.stringify(result, null, 2));
