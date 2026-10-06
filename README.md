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
`ca-certificates`. Install a PostgreSQL client to apply the database schema.

On the first checkout:

```bash
cp .env.template .env
# Configure DATABASE_URL and both JWT secrets in .env.
# Apply database/schema.sql to that database with psql once.
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
2. Apply `database/schema.sql` to the configured database once. With `DATABASE_URL`
   exported in your shell, run `psql "$DATABASE_URL" -f database/schema.sql`.
   Cargo loads `.env` for the application; `psql` does not load it automatically.
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

Apply schema manually:

```bash
psql "$DATABASE_URL" -f database/schema.sql
```

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

- Database schema is applied manually; there is no migration runner in the app startup path yet.
- External billing is reference-based only. No Stripe/PayPal/MercadoPago API integration or webhook reconciliation yet.
- Email delivery for client invites/magic links is not documented as a complete outbound mail pipeline.
- Local attachment storage requires you to back up `UPLOAD_DIR` yourself.
- S3 mode depends on provider credentials and bucket policy configured outside Horbyta.
- Tests currently focus on backend service/extractor behavior; there is no documented frontend unit/e2e suite.

## Roadmap

- First-run bootstrap that can apply migrations/schema safely.
- Proper migration history instead of a single schema file.
- External billing provider integrations and webhook reconciliation.
- Outbound email configuration for invites, magic links, and billing events.
- Broader authorization and state-machine tests across services.
- Frontend e2e coverage for core freelancer and client portal workflows.
- OpenAPI/REST documentation generated from route/DTO definitions.
- Observability guide for logs, metrics, and production health checks.

## License

All versions of this project have been publicly released under [AGPL-3.0-only](./LICENSE).

Earlier commits referencing MIT were never publicly distributed under that license.
