#!/usr/bin/env node
// Local production-build review only; deployment continues to use Nginx.
import { createServer, request as proxyRequest } from "node:http";
import { createConnection } from "node:net";
import { readFile, stat } from "node:fs/promises";
import { resolve, extname } from "node:path";
const root = resolve(process.argv[2] || "frontend/dist/pwa");
const port = Number(process.argv[3] || 9100);
const types = {
  ".html": "text/html; charset=utf-8",
  ".js": "application/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".png": "image/png",
  ".svg": "image/svg+xml",
  ".woff": "font/woff",
  ".woff2": "font/woff2",
  ".ico": "image/x-icon",
};
const server = createServer(async (req, res) => {
  if (req.url.startsWith("/api/")) {
    const upstream = proxyRequest(
      {
        hostname: "127.0.0.1",
        port: 8890,
        path: req.url,
        method: req.method,
        headers: req.headers,
      },
      (response) => {
        res.writeHead(response.statusCode, response.headers);
        response.pipe(res);
      },
    );
    upstream.on("error", () => {
      res.writeHead(502, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: "Local test backend unavailable" }));
    });
    req.pipe(upstream);
    return;
  }
  try {
    const path = new URL(req.url, "http://127.0.0.1").pathname;
    let file = resolve(root, "." + decodeURIComponent(path));
    if (file !== root && !file.startsWith(root + "/")) {
      res.writeHead(403);
      res.end();
      return;
    }
    const info = await stat(file).catch(() => null);
    if (!info?.isFile()) {
      if (extname(path)) {
        res.writeHead(404);
        res.end();
        return;
      }
      file = resolve(root, "index.html");
    }
    res.writeHead(200, {
      "Content-Type": types[extname(file)] || "application/octet-stream",
      "Cache-Control": "no-cache",
    });
    res.end(await readFile(file));
  } catch {
    res.writeHead(500);
    res.end("Build preview error");
  }
});
server.on("upgrade", (req, socket, head) => {
  if (!req.url.startsWith("/ws")) {
    socket.destroy();
    return;
  }
  const upstream = createConnection({ host: "127.0.0.1", port: 8890 }, () => {
    const headers = Object.entries(req.headers).flatMap(([name, value]) =>
      Array.isArray(value)
        ? value.map((v) => `${name}: ${v}`)
        : [`${name}: ${value}`],
    );
    upstream.write(
      `${req.method} ${req.url} HTTP/${req.httpVersion}\r\n${headers.join("\r\n")}\r\n\r\n`,
    );
    if (head.length) upstream.write(head);
    socket.pipe(upstream);
    upstream.pipe(socket);
  });
  upstream.on("error", () => socket.destroy());
  socket.on("error", () => upstream.destroy());
  socket.on("close", () => upstream.destroy());
});
server.listen(port, "127.0.0.1", () =>
  console.log(`Local build review: http://127.0.0.1:${port}`),
);
