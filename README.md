# Horbyta

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Self Hosted](https://img.shields.io/badge/Self--Hosted-Yes-success)](#horbyta)
[![Backend: Rust](https://img.shields.io/badge/Backend-Rust%202024-orange?logo=rust&logoColor=white)](#tech-stack)
[![API: Axum](https://img.shields.io/badge/API-Axum%200.8-black)](#tech-stack)
[![ORM: SeaORM](https://img.shields.io/badge/ORM-SeaORM%201.1-darkgreen)](#tech-stack)
[![Database: PostgreSQL](https://img.shields.io/badge/Database-PostgreSQL%2016%2B-336791?logo=postgresql&logoColor=white)](#database)
[![Frontend: Quasar](https://img.shields.io/badge/Frontend-Quasar%20PWA-1976D2?logo=quasar&logoColor=white)](#tech-stack)
[![Docker Compose](https://img.shields.io/badge/Docker-Compose-2496ED?logo=docker&logoColor=white)](#quick-start)
[![Tests](https://img.shields.io/badge/Tests-cargo%20test-informational)](#running-tests)
[![Storage](https://img.shields.io/badge/Storage-Local%20%7C%20S3-lightgrey)](#s3-and-attachments)
[![Auth](https://img.shields.io/badge/Auth-JWT%20Cookies-purple)](#authentication)

Horbyta is a self-hosted project management and billing platform for freelancers and independent professionals. It combines client/project management, tasks, milestones, time tracking, invoices, payments, file attachments, dashboards, and a client portal with formal change requests.

The core problem it solves is operational fragmentation: freelance work often lives across chat, spreadsheets, timers, file drives, invoice tools, and payment notes. Horbyta keeps the work, billing context, client approvals, and audit-sensitive payment records in one system you can run yourself.

## Quick Start

The repository is Docker-first. After `.env` is configured and the schema is applied, the one-command app start is:

```bash
make setup
# edit .env
psql "$DATABASE_URL" -f database/schema.sql
make prod
```

For development container workflow:

```bash
make setup
# edit .env
make dev
make shell
cargo run
```

`make dev` uses the development compose overlay:

```bash
docker compose --env-file .env -f .docker/docker-compose.yml -f .docker/docker-compose.dev.yml up --build --no-deps horbyta_app -d
```

`make prod` uses the production compose file:

```bash
docker compose --env-file .env -f .docker/docker-compose.yml up --build -d
```

The compose files live under `.docker/`, so plain `docker compose up` from the repository root is not currently the canonical command. Use the Makefile wrappers unless you intentionally pass the `-f .docker/...` files yourself.

Useful Make targets:

```bash
make help
make dev
make prod
make logs
make ps
make shell
make down
make clean
```

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

- Docker / Docker Compose
- Makefile wrappers
- Nginx sample config under `nginx/`
- Static frontend served by the Rust binary in production

## Architecture

Horbyta is a single Rust binary plus a compiled Quasar PWA.

Runtime shape:

- `src/main.rs` builds an Axum router under `/api/v1`, exposes `/ws`, and serves `STATIC_DIR` as the frontend fallback.
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

- **Self-hosted single binary.** The backend serves API, WebSocket, and static PWA assets. This keeps deployment simple for a small VPS.
- **PostgreSQL as the source of truth.** Constraints and triggers protect invariants that must survive application bugs, especially billing/payment rules.
- **Service-layer authorization.** Queries filter by `user_id` and, for the client portal, by both `user_id` and `client_id`.
- **Separate freelancer/client JWT secrets.** Freelancer and client sessions cannot be used interchangeably.
- **Cookie-based sessions.** Auth tokens are stored in HTTP cookies; frontend code does not need to manage bearer tokens manually.
- **Explicit error conventions.** The API distinguishes malformed payloads, invalid business input, state conflicts, forbidden role actions, and missing scoped resources.
- **External billing is reference-based.** Horbyta records external provider identifiers and payment links, but does not process card payments itself.
- **S3-compatible storage is optional.** Local disk is the default; S3/R2/MinIO/etc. can be enabled by environment.

## Error Conventions

Current backend convention:

| Error | Use when |
| --- | --- |
| `Unprocessable` | Request body cannot be parsed as the expected transport format, e.g. invalid JSON or multipart. |
| `BadRequest` | Payload is syntactically valid but violates DTO validation or a business input rule. |
| `Conflict` | The action is valid in general but conflicts with current state, e.g. editing a non-draft invoice or creating work inside a closed project. |
| `Forbidden` | The authenticated actor exists but is not allowed to perform that role-specific action. |
| `NotFound` | The resource does not exist in the caller's tenant scope, or should not reveal cross-tenant existence. |
| `Unauthorized` | Missing, invalid, or expired session token. |

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

1. Create `.env`:

```bash
make setup
```

2. Edit at least:

```env
DATABASE_URL=postgresql://development:developer123@host.docker.internal:5434/hormigapm_development
JWT_SECRET=replace-with-32-plus-random-characters
JWT_SECRET_CLIENT=replace-with-another-32-plus-random-string
TIME_TO_EXP_SESSION=7d
ALLOW_REGISTRATION=true
```

Use `host.docker.internal` when the app runs inside Docker and PostgreSQL runs on the host. If you apply the schema from the host shell, use the equivalent host URL, for example `postgresql://development:developer123@localhost:5434/hormigapm_development`.

3. Apply schema:

```bash
psql "$DATABASE_URL" -f database/schema.sql
```

4. Start backend container:

```bash
make dev
make logs
```

Development compose starts the Rust container in manual/sleep mode. Enter it with:

```bash
make shell
cargo run
```

For a production-style container build:

```bash
make prod
```

Manual source build:

```bash
cd frontend
npm ci
npx quasar build -m pwa
cd ..
cargo run
```

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

All runtime configuration is read from environment variables. `Config::init()` requires the critical values at startup.

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `DATABASE_URL` | yes | none | PostgreSQL connection string. |
| `JWT_SECRET` | yes | none | Secret used to sign freelancer/user JWTs. Use 32+ random chars. |
| `JWT_SECRET_CLIENT` | yes | none | Separate secret for client portal JWTs. Use 32+ random chars. |
| `TIME_TO_EXP_SESSION` | yes | none | Session duration parsed by `duration-str`, e.g. `7d`, `12h`, `30m`. |
| `APP_URI` | no | `127.0.0.1:8890` | Address the Rust server binds to. In Docker this is set to `0.0.0.0:8890`. |
| `APP_ENV` | no | `development` | `production` disables development CORS behavior. |
| `FRONTEND_ORIGIN` | dev only | `http://localhost:9000` | Allowed CORS origin outside production. |
| `STATIC_DIR` | no | `frontend/dist/pwa` | Directory served as the PWA. |
| `ALLOW_REGISTRATION` | no | `false` | Enables public user registration. Turn off after creating the first user. |
| `UPLOAD_DIR` | no | `./data/storage` | Local attachment storage path when S3 is not configured. |
| `USER_STORAGE_LIMIT_GB` | no | `2` | Per-user attachment storage cap. |
| `S3_BUCKET` | no | unset | Enables S3-compatible storage when set. |
| `S3_REGION` | no | unset | S3 region. |
| `S3_ENDPOINT` | no | unset | Custom S3 endpoint for MinIO, Cloudflare R2, Backblaze B2, etc. |
| `S3_ACCESS_KEY` | no | unset | S3 access key. |
| `S3_SECRET_KEY` | no | unset | S3 secret key. |
| `RUST_LOG` | no | `info` | Logging filter, e.g. `debug`, `info,sqlx=warn`. |
| `RUST_PORT` | compose | `8890` | Host port used by Docker Compose. |
| `APP_VERSION` | compose | `latest` | Docker image tag suffix. |
| `BINARY_NAME` | compose/build | `horbyta` | Binary copied by the production Dockerfile. |

`.env.template` includes extra helper values used by compose/development. Variables not read by the Rust app may still be used by Docker or scripts.

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
- The S3 builder uses `with_allow_http(true)`, which is useful for local MinIO but should be treated carefully in production networks.

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

- No root-level `compose.yml`; use `make dev` / `make prod` or pass the `.docker` compose files explicitly.
- The dev compose service keeps the container alive; it does not automatically run `cargo run`.
- Database schema is applied manually; there is no migration runner in the app startup path yet.
- External billing is reference-based only. No Stripe/PayPal/MercadoPago API integration or webhook reconciliation yet.
- Email delivery for client invites/magic links is not documented as a complete outbound mail pipeline.
- Local attachment storage requires you to back up `UPLOAD_DIR` yourself.
- S3 mode depends on provider credentials and bucket policy configured outside Horbyta.
- Tests currently focus on backend service/extractor behavior; there is no documented frontend unit/e2e suite.

## Roadmap

- Root-level Docker Compose profile for true `docker compose up` from the repository root.
- First-run bootstrap that can apply migrations/schema safely.
- Proper migration history instead of a single schema file.
- External billing provider integrations and webhook reconciliation.
- Outbound email configuration for invites, magic links, and billing events.
- Broader authorization and state-machine tests across services.
- Frontend e2e coverage for core freelancer and client portal workflows.
- OpenAPI/REST documentation generated from route/DTO definitions.
- Observability guide for logs, metrics, and production health checks.

## License

All versions of this project have been publicly released under AGPL-3.0-only.

Earlier commits referencing MIT were never publicly distributed under that license.
