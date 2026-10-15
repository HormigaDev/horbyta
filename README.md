# Horbyta

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Self Hosted](https://img.shields.io/badge/Self--Hosted-Yes-success)](#horbyta)
[![Backend: Rust](https://img.shields.io/badge/Backend-Rust%202024-orange?logo=rust&logoColor=white)](#tech-stack)
[![API: Axum](https://img.shields.io/badge/API-Axum%200.8-black)](#tech-stack)
[![ORM: SeaORM](https://img.shields.io/badge/ORM-SeaORM%201.1-darkgreen)](#tech-stack)
[![Database: PostgreSQL](https://img.shields.io/badge/Database-PostgreSQL%2016%2B-336791?logo=postgresql&logoColor=white)](#database)
[![Frontend: Quasar](https://img.shields.io/badge/Frontend-Quasar%20PWA-1976D2?logo=quasar&logoColor=white)](#tech-stack)
[![Tests](https://img.shields.io/badge/Tests-cargo%20test-informational)](#running-tests)
[![Storage](https://img.shields.io/badge/Storage-Local%20%7C%20S3-lightgrey)](#s3-and-attachments)
[![Auth](https://img.shields.io/badge/Auth-JWT%20Cookies-purple)](#authentication)

Horbyta is a self-hosted project management and billing platform for freelancers and independent professionals. It combines client/project management, tasks, milestones, time tracking, invoices, payments, file attachments, dashboards, and a client portal with formal change requests.

The core problem it solves is operational fragmentation: freelance work often lives across chat, spreadsheets, timers, file drives, invoice tools, and payment notes. Horbyta keeps the work, billing context, client approvals, and audit-sensitive payment records in one system you can run yourself.

## Quick Start

The backend runs directly on Linux with Cargo. Install Rust through rustup; the
repository selects Rust **1.97.1** through `rust-toolchain.toml`. A native build
also needs a C compiler/linker and `curl` (used to download Swagger UI assets).
On Debian/Ubuntu, install `build-essential`, `pkg-config`, `curl`, and
`ca-certificates`. Use the migration binary to initialize and update the database.

On the first checkout:

```bash
cp .env.template .env
# Configure DATABASE_URL and both JWT secrets in .env.
# Initialize a new database or apply pending migrations:
cargo run --locked --bin migrate
```

Then, from the repository root:

```bash
cargo check --locked
cargo build --locked
cargo run --locked
```

Plain `cargo check`, `cargo build`, and `cargo run` also work. Compilation does
not require PostgreSQL or a configured `.env`; running the backend requires a
reachable PostgreSQL database with the schema applied and valid configuration.
The app automatically loads `.env`, while exported environment variables take
precedence. Missing `.env` is allowed when the environment is already configured.
Empty S3 settings leave local attachment storage enabled.

The frontend runs independently in another terminal:

```bash
cd frontend
npm ci
npm run dev
```

Quasar runs at `http://localhost:9000` and proxies `/api` and `/ws` to the Rust
backend at `127.0.0.1:8890`. In production, Nginx serves the compiled PWA and
proxies API/WebSocket requests to the backend.

## Tech Stack

Backend:

- Rust 2024
- Axum 0.8
- SeaORM 1.1
- PostgreSQL 16+
- JWT cookies for freelancer and client sessions
- Argon2 password hashing
- object_store for local/S3-compatible attachments
- WebSocket endpoint for realtime notifications

Frontend:

- Vue 3
- Quasar PWA
- TypeScript
- i18n: English, Spanish, Portuguese

Infrastructure:

- Native Rust binary supervised by systemd in production
- Nginx sample config under `nginx/`
- Static frontend served by Nginx in production

## Architecture

Horbyta is a Rust backend binary plus an independently built Quasar PWA.

Runtime shape:

- `src/main.rs` builds an Axum router under `/api/v1` and exposes `/ws`.
- Nginx serves the compiled PWA with a history-mode SPA fallback; the Rust backend does not serve frontend files.
- Controllers in `src/controllers/` handle HTTP extraction, request parsing, and response shape.
- Services in `src/services/` own business rules, transactions, multi-tenant scoping, and persistence orchestration.
- DTOs in `src/dtos/` define API payloads, validation, search fields, and response projections.
- Middlewares/extractors in `src/middlewares/` handle auth, client auth, validation, and rate limiting helpers.
- SeaORM entities in `src/entities/` map the PostgreSQL schema.
- `database/schema.sql` contains tables, constraints, indexes, views, and triggers.

Important flows:

- Freelancer routes authenticate through `UserDto` extractor and `horbyta_user_auth` cookie.
- Client portal routes authenticate through `ClientAuthDto` extractor and `horbyta_client_auth` cookie.
- Search endpoints use validated JSON encoded in the `q` query parameter.
- Attachments are scoped to either a project or a task, never both.
- Project creation with files is parsed at the controller layer and orchestrated by `ProjectService::create_with_attachments`.

## Technical Decisions

- **Self-hosted native backend.** One Rust binary serves the API and WebSocket endpoint. Nginx serves the static PWA assets on the VPS.
- **PostgreSQL as the source of truth.** Constraints and triggers protect invariants that must survive application bugs, especially billing/payment rules.
- **Service-layer authorization.** Queries filter by `user_id` and, for the client portal, by both `user_id` and `client_id`.
- **Separate freelancer/client JWT secrets.** Freelancer and client sessions cannot be used interchangeably.
- **Cookie-based sessions.** Auth tokens are stored in HTTP cookies; frontend code does not need to manage bearer tokens manually.
- **Explicit error conventions.** The API distinguishes malformed payloads, invalid business input, state conflicts, forbidden role actions, and missing scoped resources.
- **External billing is reference-based.** Horbyta records external provider identifiers and payment links, but does not process card payments itself.
- **S3-compatible storage is optional.** Local disk is the default; S3/R2/MinIO/etc. can be enabled by environment.

## Error Conventions

Current backend convention:

| Error           | Use when                                                                                                                                    |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `Unprocessable` | Request body cannot be parsed as the expected transport format, e.g. invalid JSON or multipart.                                             |
| `BadRequest`    | Payload is syntactically valid but violates DTO validation or a business input rule.                                                        |
| `Conflict`      | The action is valid in general but conflicts with current state, e.g. editing a non-draft invoice or creating work inside a closed project. |
| `Forbidden`     | The authenticated actor exists but is not allowed to perform that role-specific action.                                                     |
| `NotFound`      | The resource does not exist in the caller's tenant scope, or should not reveal cross-tenant existence.                                      |
| `Unauthorized`  | Missing, invalid, or expired session token.                                                                                                 |

Responses use:

```json
{
    "error": "human readable message",
    "type": "bad_request"
}
```

## State Rules

Project status affects downstream creation:

- Closed projects cannot accept new tasks.
- Closed projects cannot accept new milestones.
- Closed projects cannot accept new invoices.
- Closed projects cannot accept new change requests.

Invoice status rules:

- Only `draft` invoices can be edited or deleted.
- Allowed status transitions are validated in the invoice service.
- Payments can only be recorded for invoices in compatible states.

Payment rules:

- Payments are immutable for sensitive fields.
- Refunds are represented as separate payment records linked to the original charge.
- Database triggers reject tampering and invalid refund logic.

## Running Locally

1. Copy `.env.template` to `.env` if it does not exist. Configure at least
   `DATABASE_URL`, `JWT_SECRET`, `JWT_SECRET_CLIENT`, and `TIME_TO_EXP_SESSION`.
   Use the PostgreSQL host reachable from your machine, for example `localhost`.
2. Run `cargo run --locked --bin migrate`. The first migration reads the base
   `database/schema.sql` automatically; new databases need no separate psql step.
   Later releases add incremental migrations; never reapply the schema manually.
3. Run `cargo run` in the repository root.
4. In another terminal, run `npm ci` and `npm run dev` inside `frontend/`.

Stop the backend with Ctrl+C. The backend also handles SIGTERM for systemd stops.
The default local attachment directory is `data/storage/`; it is created at
startup. No frontend build is required to run or check the backend.

## Native VPS Deployment

Build the backend on a compatible Linux system and build the PWA separately:

```bash
cargo build --release --locked
cd frontend
npm ci
npm run build
```

Deploy `target/release/horbyta` to `/opt/horbyta/bin/horbyta` and the contents of
`frontend/dist/pwa/` to `/opt/horbyta/frontend/dist/pwa/`. Neither Rust nor Node
is required on the VPS when deploying prebuilt artifacts. The binary must match
the VPS architecture and system-library compatibility; it is not guaranteed to
be fully statically linked.

A sample service is provided in `deploy/horbyta.service`. Before enabling it:

- Create a dedicated system user and group named `horbyta`.
- Create `/etc/horbyta/horbyta.env` with the database URL, JWT secrets, session
  duration, and public `FRONTEND_ORIGIN`. Use production values, including
  `APP_ENV=production`, `APP_URI=127.0.0.1:8890`, and
  `UPLOAD_DIR=/var/lib/horbyta/storage`. Restrict access to this file.
- Ensure PostgreSQL is available and apply the schema before first startup.
- Install the service file at `/etc/systemd/system/horbyta.service`, then run
  `sudo systemctl daemon-reload` and `sudo systemctl enable --now horbyta`.

The service creates its persistent storage directory through `StateDirectory`,
restarts after failures, and sends application logs to the journal:
`journalctl -u horbyta -f`. Variables in the environment file override the
service's default environment settings.

Configure Nginx using the files under `nginx/`, replace the domain placeholder,
and configure HTTPS for the public domain. Nginx serves the PWA and proxies
`/api/` and `/ws` to `127.0.0.1:8890`. Back up both PostgreSQL and the local
attachment directory, or use S3 for attachments.

## Create an Administrator

The auxiliary CLI connects directly to PostgreSQL using `DATABASE_URL` from
`.env` or the process environment. It creates an **active admin**, independently
of `ALLOW_REGISTRATION`, and shares the application's validation, uniqueness
checks and Argon2 password hashing. It does not require JWT configuration or a
running HTTP server, and never overwrites an existing account.

Interactive mode asks for email and a hidden password with confirmation:

```bash
cargo run --bin create-user
```

Arguments are also supported; `--` separates Cargo options from CLI options:

```bash
cargo run --bin create-user -- admin@example.com 'YourPassword123!' --name 'Admin'
```

Omit the password argument to prompt for it. The interactive mode keeps the
password out of command history and the process argument list. The display name
defaults to the email prefix; use `--name` if that name is already taken.
Automation can provide one password line via stdin:

```bash
cargo run --bin create-user -- admin@example.com --password-stdin --name 'Admin'
```

`cargo run` still starts the backend by default. Invalid input, duplicate
accounts and database failures return a nonzero exit status. `--help` works
without database configuration.

## Headless UI Screenshots

`scripts/capture-ui.mjs` uses local Chrome in headless mode through an isolated
DevTools pipe and temporary browser profile. It opens no visible browser window
and does not use your personal browser profile. See
[Chrome's headless documentation](https://developer.chrome.com/docs/automation-and-testing/headless).

Start the backend (`cargo run`) and the independent frontend (`npm run dev` in
`frontend/`), then run from the repository root:

```bash
node scripts/capture-ui.mjs http://127.0.0.1:9000
# Or capture selected routes:
node scripts/capture-ui.mjs http://127.0.0.1:9000 /tasks /board /settings
```

Provide `UI_TEST_EMAIL` and `UI_TEST_PASSWORD` in the process environment, or a
local `target/ui-tests/account.json` file containing `email` and `password`.
Test credentials must belong to a dedicated account created with the CLI.
Credentials and screenshots under `target/` are excluded from version control;
restrict the credentials file to its owner (`chmod 600`).

The script signs in through the login form and captures dashboard, tasks,
projects, board, new-task form and settings at desktop and mobile sizes. PNGs
and a report with route, viewport, theme, overflow measurements and runtime/HTTP
diagnostics are written to `target/ui-tests/screenshots/`. It does not seed
projects/tasks or change preferences. Chrome's temporary profile is removed
when the script finishes. No npm automation dependency is required.

Overrides: `CHROME_BIN`, `UI_SCREENSHOT_DIR`. Some restricted environments require
`CHROME_NO_SANDBOX=1`; ordinary local runs keep Chrome's sandbox enabled.
`node scripts/capture-ui.mjs --help` lists all options.

## Running Tests

Backend tests:

```bash
cargo test
```

Compile tests without running:

```bash
cargo test --no-run
```

Target one module/test:

```bash
cargo test services::projects
cargo test normalize_billing_payload
```

Frontend static checks depend on the frontend package scripts:

```bash
cd frontend
npm ci
npm run lint
```

## Environment Variables

Runtime configuration is read from environment variables, with `.env` loaded at startup without overriding existing variables. `Config::init()` requires the critical values at startup.

| Variable                | Required      | Default                 | Description                                                                |
| ----------------------- | ------------- | ----------------------- | -------------------------------------------------------------------------- |
| `DATABASE_URL`          | yes           | none                    | PostgreSQL connection string.                                              |
| `JWT_SECRET`            | yes           | none                    | Secret used to sign freelancer/user JWTs. Use 32+ random chars.            |
| `JWT_SECRET_CLIENT`     | yes           | none                    | Separate secret for client portal JWTs. Use 32+ random chars.              |
| `TIME_TO_EXP_SESSION`   | yes           | none                    | Session duration parsed by `duration-str`, e.g. `7d`, `12h`, `30m`.        |
| `APP_URI`               | no            | `127.0.0.1:8890`        | Address the Rust server binds to; use loopback behind Nginx. |
| `APP_ENV`               | no            | `development`           | `production` disables Swagger UI; configure the frontend origin for the deployed domain.                           |
| `FRONTEND_ORIGIN`       | no            | `http://localhost:9000` | Allowed browser origin for CORS and origin checks.                                    |
| `ALLOW_REGISTRATION`    | no            | `false`                 | Enables public user registration. Turn off after creating the first user.  |
| `UPLOAD_DIR`            | no            | `./data/storage`        | Local attachment storage path when S3 is not configured.                   |
| `USER_STORAGE_LIMIT_GB` | no            | `2`                     | Per-user attachment storage cap.                                           |
| `S3_BUCKET`             | no            | unset                   | Enables S3-compatible storage when set.                                    |
| `S3_REGION`             | no            | unset                   | S3 region.                                                                 |
| `S3_ENDPOINT`           | no            | unset                   | Custom S3 endpoint for MinIO, Cloudflare R2, Backblaze B2, etc.            |
| `S3_ACCESS_KEY`         | no            | unset                   | S3 access key.                                                             |
| `S3_SECRET_KEY`         | no            | unset                   | S3 secret key.                                                             |
| `RUST_LOG`              | no            | `info`                  | Logging filter, e.g. `debug`, `info,sqlx=warn`.                            |

`.env.template` contains application settings. Empty optional S3 values are treated as unset.

## Authentication

There are two independent auth flows.

Freelancer/user auth:

- Login endpoint validates email/password.
- Passwords are hashed with Argon2.
- JWT claims contain `sub` as the user id and `exp`.
- Token is stored in `horbyta_user_auth`.
- Protected freelancer endpoints extract `UserDto` from the cookie.
- The extractor verifies JWT signature with `JWT_SECRET`, then loads/caches the user.

Client portal auth:

- Clients can authenticate with credentials or magic links.
- Client JWT claims contain `sub` as the `client_auth.id`.
- Token is stored in `horbyta_client_auth`.
- Protected client endpoints extract `ClientAuthDto` from the cookie.
- The extractor verifies JWT signature with `JWT_SECRET_CLIENT`, then loads/caches the linked client profile.

Security properties:

- User cookies do not authenticate client routes.
- Client cookies do not authenticate freelancer routes.
- Invalid or missing cookies return `401 Unauthorized`.
- Tenant access is enforced by service queries using `user_id` and, for client routes, `client_id`.

## S3 And Attachments

Horbyta uses the `object_store` abstraction.

Default behavior:

- If `S3_BUCKET` is not set, files are stored in `UPLOAD_DIR`.
- The app creates the local upload directory at startup if needed.
- Attachments are metadata rows in PostgreSQL plus binary objects in local/S3 storage.

S3-compatible behavior:

- Set `S3_BUCKET` to enable S3 mode.
- Set `S3_REGION`, `S3_ACCESS_KEY`, `S3_SECRET_KEY`.
- Set `S3_ENDPOINT` for non-AWS providers such as MinIO, Cloudflare R2, or Backblaze B2.
- Plain HTTP S3 endpoints are allowed only outside production; production requires HTTPS.

Attachment safeguards:

- A file must be attached to exactly one scope: project or task.
- Per-file and per-scope size checks are enforced by the attachment service.
- Per-user storage is capped by `USER_STORAGE_LIMIT_GB`.

## External Billing

Horbyta does not currently execute payments through Stripe/PayPal/etc. It records billing state and external references.

Project billing:

- `fixed`: requires `fixed_price`.
- `hourly`: requires `hourly_rate`.
- `retainer`: requires `retainer_amount` and `payment_day`.
- Currency is normalized to uppercase.

Invoices:

- Providers: `manual`, `paypal`, `stripe`, `wise`, `mercadopago`, `revolut`, `payoneer`.
- If provider is not `manual`, `provider_external_id` is required.
- `payment_link` can store a provider checkout/payment URL.
- External provider ids are unique per provider when present.

Payments:

- Manual charges and refunds are recorded as internal payment records.
- Refunds point to their original payment.
- Provider fields exist for external references, but provider reconciliation/webhooks are not implemented yet.

## Database

Initialize or update the schema:

```bash
cargo run --locked --bin migrate
```

Migration 1 embeds the original `database/schema.sql`; migration 2 combines the
previous scope and negotiation extensions in one additional file.
The runner records its checksum and executes it atomically under an advisory lock.
Already initialized databases receive only unapplied migrations.

The schema includes:

- UUID primary keys.
- Record numbers scoped by user/table.
- Check constraints for enum-like fields.
- Billing consistency constraints.
- Immutable payment triggers.
- Invoice/payment guard triggers.
- Dashboard views.

## Nginx

Sample config lives in `nginx/`.

```bash
sudo cp nginx/http-directives.conf /etc/nginx/conf.d/pm-http-directives.conf
sudo cp nginx/pm-api.conf /etc/nginx/sites-available/pm-api
sudo ln -s /etc/nginx/sites-available/pm-api /etc/nginx/sites-enabled/pm-api
sudo nginx -t
sudo systemctl reload nginx
```

The provided config proxies to `127.0.0.1:8890` and includes WebSocket upgrade headers.

## Known Limitations

- Migrations run explicitly with `cargo run --locked --bin migrate`; this includes first-run initialization. Startup checks the required schema version.
- External billing is reference-based only. No Stripe/PayPal/MercadoPago API integration or webhook reconciliation yet.
- Email delivery for client invites/magic links is not documented as a complete outbound mail pipeline.
- Local attachment storage requires you to back up `UPLOAD_DIR` yourself.
- S3 mode depends on provider credentials and bucket policy configured outside Horbyta.
- Tests currently focus on backend service/extractor behavior; there is no documented frontend unit/e2e suite.

## Roadmap

- External billing provider integrations and webhook reconciliation.
- Outbound email configuration for invites, magic links, and billing events.
- Broader authorization and state-machine tests across services.
- Frontend e2e coverage for core freelancer and client portal workflows.
- OpenAPI/REST documentation generated from route/DTO definitions.
- Observability guide for logs, metrics, and production health checks.

## License

All versions of this project have been publicly released under [AGPL-3.0-only](./LICENSE).

Earlier commits referencing MIT were never publicly distributed under that license.

## Laptop resource limits

Cargo uses **one job** and one Rayon, Tokio and test worker through
`.cargo/config.toml`. Frontend lint runs directly with `npm run lint`.
For laptop backend checks, apply Linux limits directly, without a wrapper or
available-memory gate:

```sh
systemd-run --user --scope -p CPUQuota=50% -p MemoryMax=1536M -p MemorySwapMax=0 nice -n 19 cargo check --locked
```

The same prefix applies to `cargo test` and builds; use `--test-threads=1` for tests.
Run backend and frontend builds sequentially.
UI verification can run against the dedicated test account with backend and
Quasar running: `UI_VERIFY=1 node scripts/capture-ui.mjs`. It requires representative
test tasks and Spanish preferences, exercises preview/cancel, failed saves,
live system appearance, explicit record actions and mobile layout. It writes
`verification.json` beside the screenshots. Credentials and test artifacts stay
under the ignored `target/ui-tests/` directory.

`UI_VERIFY_FORMS=1` with the same capture command checks draft restoration,
failed presets and task create/edit through the real form. It removes the temporary
task afterward and only accepts the dedicated UI test account.

`UI_VERIFY_WORKFLOWS=1` also exercises keyboard focus, pagination, filters,
column persistence, financial forms with regional formats and attachment storage.
It uses and deletes temporary records in the dedicated test account and restores
its table/regional preferences. Use a writable `UPLOAD_DIR` for the test backend;
existing storage owned by another OS user must have its ownership fixed separately.

`UI_VERIFY_DASHBOARD_PORTAL=1` verifies chart chronology/currencies, accessible
chart tables, refresh/filter failures, actual disposable client credentials/token
login and portal isolation. Financial data and 2FA challenges are browser-only
fixtures; client records are created through the real API and deleted afterward.
It requires PostgreSQL, the backend, Quasar and the dedicated test account.

`UI_VERIFY_CLIENT_STATES=1` checks client details, ES/EN/PT layouts, request
status locks, leave guards and draft restoration using isolated browser fixtures.
It needs Quasar and Chrome but does not authenticate against the backend or test
server permissions. Keep this mode separate from the real-account flags above.
Choose a separate `UI_SCREENSHOT_DIR` to preserve evidence from previous runs.

`UI_VERIFY_MIGRATION=1` runs the final appearance matrix (96 list/form
combinations at 320/360/768/1440px), page families, keyboard/touch emulation,
regional display and real A/B account isolation. It creates disposable records
and a temporary second user, then restores preferences and removes those records.
Use the dedicated account and the resource wrapper; reports go to the selected
`UI_SCREENSHOT_DIR`. Full blocked-storage behavior is checked on production:
development Devtools itself assumes browser storage.

For local production-build review, after `npm run build`, run
`node scripts/serve-ui-build.mjs` from the
repository root, with the test backend on port 8890. Then use
`UI_VERIFY_PRODUCTION=1 node scripts/capture-ui.mjs http://127.0.0.1:9100 / /tasks /settings`.
The preview server binds localhost and proxies the test API; VPS hosting remains
Nginx. Production checks cover real login, PWA shell, local fonts, storage errors
and modal keyboard focus. Offline business operations are not provided.

`node scripts/measure-ui-build.mjs frontend/dist/pwa` reports static entry/import
sizes and total assets. It separates the initial static graph from dynamic
imports; PWA precaching may still download lazy chunks in the background.
Pass a capture `report.json` as a third argument to also measure the observed
login imports and their static dependencies. The shared component/token contracts
are documented in [frontend/DESIGN.md](frontend/DESIGN.md).

## Control formal de alcance

El alcance original se publica y acepta desde el detalle del proyecto. Las tareas
posteriores se clasifican en su detalle como incluidas o adicionales de un cambio.
Los extras permanecen bloqueados hasta aprobar su revisión exacta. La aprobación
genera un suplemento fijo en una factura nueva en borrador y ajusta únicamente
el hito indicado; las horas del extra se registran como no facturables para evitar
duplicar el cobro. El portal muestra revisiones, aclaraciones y decisiones pendientes.

Ver [contratos, migraciones y conservación](docs/scope-control.md). Los acuerdos
se conservan al cerrar/cancelar; las eliminaciones que destruirían evidencia se
rechazan. Los antecedentes migrados no reciben aceptaciones ni efectos inventados.

Validación con PostgreSQL, siempre en un esquema aislado que se elimina al terminar:

```bash
cargo test --locked --test scope_control -- --ignored --test-threads=1
```

Puede usarse `SCOPE_TEST_DATABASE_URL` para indicar una base de pruebas diferente.
La cuenta de conexión necesita permiso para crear un esquema; las pruebas no
actualizan las tablas habituales. `KEEP_SCOPE_TEST_SCHEMA=1` conserva el esquema
de laboratorio para la validación browserless; no usar esa opción en producción.

Para revisar el portal contra un PostgreSQL de laboratorio accesible (sin reutilizar
cuentas reales), indicar **la misma** `SCOPE_TEST_DATABASE_URL` en la creación del
esquema y al iniciar el backend auxiliar:

```bash
export SCOPE_TEST_DATABASE_URL='postgresql://scope_test@127.0.0.1:55439/postgres'
KEEP_SCOPE_TEST_SCHEMA=1 cargo test --locked --test scope_control -- --ignored --test-threads=1
cargo build --locked --bin horbyta
python3 scripts/start-scope-test-server.py
```

El helper inicia el API en `127.0.0.1:8897` con secretos temporales y el esquema
identificado en `target/scope-tests/state.json`; sin la variable de conexión usa
el laboratorio local `postgresql://scope_test@127.0.0.1:55439/postgres`.
En otras terminales, tras construir Quasar en `frontend`:

```bash
UI_API_PORT=8897 node scripts/serve-ui-build.mjs frontend/dist/pwa 9107
UI_VERIFY_SCOPE=1 UI_SCREENSHOT_DIR=target/scope-tests/screenshots node scripts/capture-ui.mjs http://127.0.0.1:9107 / /projects /tasks
```

Chrome opera sin ventana ni perfil personal. El recorrido genera sus proyectos y
cambios, admite repetición y escribe evidencia en `scope-verification.json`.
Al terminar, detener los dos servidores y eliminar únicamente el esquema
`scope_test_…` creado para ese laboratorio. La validación normal sin `KEEP` lo
elimina automáticamente. El ensayo adicional de backup local se describe en
[docs/scope-control.md](docs/scope-control.md).


El alcance documental permite importar Markdown completo o requisitos separados,
elegir anexos incluidos, conservar originales/versiones/hashes y negociar ajustes,
rechazo o preferencia por una revisión histórica. Los proyectos nuevos requieren
acuerdo antes de ejecutar; las tareas de preparación se justifican y son no
facturables. La única ampliación consolidada de alcance incluye también la negociación documental. La aceptación
registra IP del socket; detrás de Nginx configurar `TRUSTED_PROXY_IPS=127.0.0.1,::1`
y mantener cerrado el acceso directo al API. Ver el procedimiento y reglas en
[docs/scope-control.md](docs/scope-control.md).


## Instalación self-hosted y datos de desarrollo

La instalación vacía y las futuras actualizaciones usan el mismo comando:
`cargo run --locked --bin migrate`, o `./migrate` al distribuir binarios compilados.
La migración **1** apunta a `database/schema.sql`, que conserva el esquema base
original; incluye compatibilidad UUIDv7 para PostgreSQL 16/17 mediante
`database/uuidv7.sql` y usa la función nativa cuando está disponible. La migración
**2**, `database/migrations/002_scope_control.sql`, reúne las dos ampliaciones
anteriores de alcance y negociación en **una sola migración adicional**.
La migración **3**, `database/migrations/003_scope_conversation.sql`, añade el
índice de paginación y los adjuntos inmutables de conversaciones. No altera las
migraciones 1 y 2.

No ejecutar `schema.sql` sobre una base existente: sus `CREATE TABLE` darán errores.
Si la base ya contiene el esquema base sin historial, `migrate` compara tablas,
columnas, restricciones, índices, vistas, funciones y triggers con una copia
transaccional del esquema original. Solo si coincide registra el paso base como
aplicado y ejecuta la ampliación pendiente, conservando los datos. Una estructura
incompatible se rechaza con la operación revertida. Esta comprobación necesita
permiso para crear un esquema temporal en la misma base.

Se consolidó el historial experimental antes del lanzamiento. Bases con los
checksums de aquel historial no se reetiquetan silenciosamente; necesitan una
transición explícita. En desarrollo autorizado se reconstruyen con el seed.
La migración **4**, `database/migrations/004_change_conversation.sql`, añade el índice
para consultar mensajes de solicitudes de cambio por revisión y los adjuntos
inmutables de esos mensajes. Se aplica con `cargo run --bin migrate` sin reescribir
las migraciones anteriores ni alterar `database/schema.sql`.

La migración **5**, `database/migrations/005_project_conversation.sql`, unifica
la conversación del proyecto mediante una vista sobre el historial existente y
mensajes nuevos. Añade lecturas privadas persistentes y vínculos entre solicitudes
y sus mensajes originales. No modifica los acuerdos ni las migraciones anteriores.

La migración **6**, `database/migrations/006_scope_amendments.sql`, permite proponer
correcciones de acuerdos aceptados y consolida automáticamente los cambios aprobados.
Conserva firmas y suplementos anteriores; no repite facturas ni fabrica aceptaciones.
También incorpora los cambios históricos que disponen de efectos verificables.

Conservar los SQL y checksums de las migraciones aplicadas (actualmente 1–6);
añadir nuevas versiones para futuras ampliaciones. El ejecutor lleva los SQL embebidos: no requiere Rust ni archivos SQL
externos en el servidor. La instalación real crea su administrador con `./create-user`.

El seed es **opcional y exclusivo de desarrollo**. Exige `APP_ENV=development`,
base `hormigapm_development` y almacenamiento local (`UPLOAD_DIR`, sin S3).
No se ejecuta desde `migrate` ni desde el backend. Sin `--reset` requiere una base
sin usuarios. Para regenerar datos ficticios de esta base:

```bash
cargo run --locked --bin seed -- --reset
```

`--reset` sustituye el esquema `public` de desarrollo y vuelve a aplicar migración
más seed dentro de una transacción. Hacer backup antes de regenerarlo si hay datos
que interesa conservar. Mantiene archivos anteriores ajenos al seed; sustituye
únicamente archivos con prefijo `development-seed/` de un seed anterior.

- Administrador: **admin@horbyta.test / Admin@123**.
- Clientes del portal: **cliente1@horbyta.test** hasta **cliente8@horbyta.test**,
  contraseña **Cliente@123**. El portal también pide el ID del administrador,
  que puede consultarse desde la cuenta o en PostgreSQL.
- 12 proyectos, clientes activos/inactivos/bloqueados, categorías y etiquetas,
  tareas y subtareas, hitos, tiempo de los últimos 90 días, archivos reales de ejemplo,
  facturas en distintos estados, cargos y reembolsos en EUR/USD/BRL.
- Alcances en borrador, pendientes, con ajustes/rechazo y aceptados; documentos
  versionados, requisitos vinculados, comentarios y solicitudes de cambio en
  todos los estados. La aprobación produce sus efectos mediante los disparadores
  normales, sin desactivarlos ni fabricar registros finales.
- Notificaciones leídas/pendientes, preferencias, estado de interfaz y conversación
  ficticia. Fechas relativas al día de ejecución mantienen utilidad visual.

Pruebas aisladas del seed: `cargo test --locked --test development_seed -- --ignored
--test-threads=1`. La actualización del esquema antiguo se comprueba con
`cargo test --locked --test migration_upgrade -- --ignored --test-threads=1`. Requieren conexión a la base de desarrollo; crean y eliminan
su propio esquema. No vacían `public`. Las credenciales de ejemplo nunca se instalan
como parte de una instalación de producción.


Para preparar los binarios de una distribución de producción (sin el seed),
construir para la arquitectura y sistema soportados:

```bash
cargo build --release --locked --bin horbyta --bin migrate --bin create-user
```

Entregar esos tres binarios, la configuración de ejemplo, el build Quasar servido
por Nginx y las instrucciones de instalación. No incluir `.env`, `data/storage`,
`target/ui-tests`, dumps ni resultados de desarrollo. El cliente configura su
conexión y secretos, ejecuta `migrate`, crea su administrador con `create-user` y
arranca `horbyta`. No necesita Rust para ejecutar los binarios.


La reconstrucción realizada en desarrollo conserva un respaldo previo completo en
`target/development-seed/before-bootstrap-reset.dump` (privado, ignorado por Git).
La validación headless del seed usa `UI_VERIFY_SEED=1` en `scripts/capture-ui.mjs`,
con `target/ui-tests/account.json` generado para la cuenta ficticia. Solo consulta
listas/documentos y comprueba los accesos; no crea proyectos para esta verificación.
