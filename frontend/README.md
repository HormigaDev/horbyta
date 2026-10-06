# Horbyta frontend

The Quasar PWA runs independently from the Rust backend during development.
Install a Node version supported by `package.json` (Node 22.12+ within the 22.x
series, or a supported newer major) and use npm with the committed lockfile.

```bash
npm ci
npm run dev
```

The dev server runs at `http://localhost:9000` and proxies `/api` and `/ws` to
`127.0.0.1:8890`. Start the backend with `cargo run` in the repository root in
another terminal.

```bash
npm run lint
npm run build
```

The production build is written to `dist/pwa/`. Deploy it to
`/opt/horbyta/frontend/dist/pwa/` on the VPS; Nginx serves these files and proxies
API/WebSocket requests to the backend. See the root README and `nginx/` configs.

```bash
npm run format
```
