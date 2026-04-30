# Horbyta

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Self-Hosted](https://img.shields.io/badge/Self--Hosted-Yes-success)](#installation)
[![Backend: Rust](https://img.shields.io/badge/Backend-Rust-orange?logo=rust&logoColor=white)](#tech-stack)
[![Frontend: Quasar](https://img.shields.io/badge/Frontend-Quasar%20PWA-1976D2?logo=quasar&logoColor=white)](#tech-stack)
[![Database: PostgreSQL](https://img.shields.io/badge/Database-PostgreSQL-336791?logo=postgresql&logoColor=white)](#requirements)
[![Docker Ready](https://img.shields.io/badge/Docker-Ready-2496ED?logo=docker&logoColor=white)](#option-c--docker-maximum-version-safety)
[![PWA](https://img.shields.io/badge/PWA-Installable-5A0FC8)](#using-horbyta)
[![i18n](https://img.shields.io/badge/i18n-EN%20%7C%20ES%20%7C%20PT-informational)](#using-horbyta)
[![API: REST](https://img.shields.io/badge/API-REST%20%2B%20WebSocket-informational)](#tech-stack)

**Horbyta** is a self-hosted project management platform built specifically for **freelancers and independent professionals**. It brings together everything you need to run your client work from a single place: projects, tasks, milestones, time tracking, invoices, payments, a client portal, and a formal change-request engine designed to stop scope creep before it eats your margin.

You own your data, you run it on your own infrastructure, and you only pay for the VPS.

---

## Table of Contents

- [What you can do with Horbyta](#what-you-can-do-with-horbyta)
- [Tech stack](#tech-stack)
- [Requirements](#requirements)
- [Installation](#installation)
    - [Option A — Production (recommended): use a release](#option-a--production-recommended-use-a-release)
    - [Option B — Manual install from source](#option-b--manual-install-from-source)
    - [Option C — Docker (maximum version safety)](#option-c--docker-maximum-version-safety)
- [Configuration reference](#configuration-reference)
- [Putting it behind Nginx (HTTPS + rate limiting)](#putting-it-behind-nginx-https--rate-limiting)
- [First run](#first-run)
- [Using Horbyta](#using-horbyta)
- [Backups](#backups)
- [Updating](#updating)
- [Troubleshooting](#troubleshooting)
- [License](#license)

---

## What you can do with Horbyta

[![Projects](https://img.shields.io/badge/Projects-Categories%20%7C%20Status%20%7C%20Billing-blueviolet)](#)
[![Tasks](https://img.shields.io/badge/Tasks-Hierarchy%20%7C%20Tags%20%7C%20Priority-informational)](#)
[![Milestones](https://img.shields.io/badge/Milestones-Tracking%20%7C%20Due%20Dates-9cf)](#)
[![Time Tracking](https://img.shields.io/badge/Time%20Tracking-Per%20Activity%20%7C%20Daily%20Limits-orange)](#)
[![Invoices](https://img.shields.io/badge/Invoices-Manual%20%7C%20Provider%20Refs-success)](#)
[![Payments](https://img.shields.io/badge/Payments-Charges%20%2B%20Refunds-success)](#)
[![Change Requests](https://img.shields.io/badge/Change%20Requests-Anti%20Scope%20Creep-critical)](#)
[![Client Portal](https://img.shields.io/badge/Client%20Portal-Magic%20Link%20%7C%20Credentials-yellowgreen)](#)
[![Attachments](https://img.shields.io/badge/Attachments-Local%20%7C%20S3-lightgrey)](#)
[![Dashboard](https://img.shields.io/badge/Dashboard-Revenue%20%7C%20AR%20%7C%20Bottlenecks-blue)](#)

- **Clients & Projects.** Group your work by client, organize projects in categories, and configure billing per project: fixed price, hourly rate, or monthly retainer.
- **Tasks with real hierarchy.** Unlimited subtasks, priority and complexity scoring, tags, due dates, and per-task time tracking.
- **Milestones.** Define checkpoints with due dates, track progress (open / review / approved / released), and surface what is overdue or upcoming.
- **Time tracking.** Start/stop timers, log entries by activity (Development, Design, Meeting, …), set a daily-hour ceiling per user, and roll up billable vs non-billable hours.
- **Invoices & Payments.** Issue invoices in any currency, attach time entries, register charges and refunds, and prevent accidental tampering — payment records are protected by database triggers.
- **Anti-scope-creep change requests.** When a client asks for "just one more thing", turn it into a formal request with technical assessment, cost impact, and schedule impact. The client must explicitly approve before work proceeds. No more arguing over chat history.
- **Client portal.** Invite clients via email + password or a magic link. They see their projects, milestones, and pending change requests.
- **Attachments.** Upload files to projects or tasks, stored locally or on any S3-compatible bucket (AWS S3, MinIO, Cloudflare R2, Backblaze B2…).
- **Real-time updates.** WebSocket-based notifications keep the UI fresh.
- **Multilingual.** Full UI translation in English, Spanish, and Portuguese.
- **Dashboards.** Monthly revenue, accounts receivable by client, billable ratio, time by activity, scope-creep value captured, task bottlenecks, and per-client portfolio views.

---

## Tech stack

[![Rust](https://img.shields.io/badge/Rust-1.88%2B-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.8-black)](https://github.com/tokio-rs/axum)
[![SeaORM](https://img.shields.io/badge/SeaORM-1.1-darkgreen)](https://www.sea-ql.org/SeaORM/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16%2B-336791?logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![Quasar](https://img.shields.io/badge/Quasar-Vue%203-1976D2?logo=quasar&logoColor=white)](https://quasar.dev/)
[![JWT](https://img.shields.io/badge/Auth-JWT-purple)](https://jwt.io/)
[![Argon2](https://img.shields.io/badge/Hashing-Argon2-success)](#)

The backend is a single Rust binary (Axum + SeaORM) that serves the REST API, the WebSocket endpoint, and the compiled frontend (Quasar PWA) as static files. PostgreSQL is the only required external service.

---

## Requirements

| Component             | Minimum                  | Notes                                                 |
| --------------------- | ------------------------ | ----------------------------------------------------- |
| OS                    | Linux x86_64             | Tested on Debian / Ubuntu                             |
| RAM                   | 512 MB                   | The binary runs comfortably under a 512 MB cap        |
| Disk                  | 1 GB + your attachments  | Attachments may be on S3 instead                      |
| PostgreSQL            | 16 or newer              | Requires the `uuidv7()` function (built-in in PG 16+) |
| Reverse proxy         | Optional but recommended | Nginx config provided                                 |
| S3-compatible storage | Optional                 | If unset, files are stored on local disk              |

---

## Installation

You have two paths. Pick one.

### Option A — Production (recommended): use a release

The fastest way to deploy Horbyta on a VPS. Each GitHub release ships a tarball with the prebuilt Rust binary plus the compiled frontend. No Rust toolchain, no Node.js, no build step on your server.

**1. Provision PostgreSQL** and create the database:

```bash
sudo -u postgres psql <<'SQL'
CREATE USER horbyta WITH PASSWORD 'change-me-strong-password';
CREATE DATABASE horbyta OWNER horbyta;
SQL
```

**2. Download the latest release** from the [Releases page](https://github.com/HormigaDev/horbyta/releases) and unpack it:

```bash
sudo mkdir -p /opt/horbyta
cd /opt/horbyta
sudo curl -L -o horbyta.tar.gz https://github.com/HormigaDev/horbyta/releases/latest/download/horbyta-linux-x86_64.tar.gz
sudo tar -xzf horbyta.tar.gz
sudo rm horbyta.tar.gz
```

After unpacking you will have, at minimum, the `horbyta` binary, the compiled `frontend/dist/pwa/` directory, and the SQL schema.

**3. Apply the database schema:**

```bash
psql "postgresql://horbyta:change-me-strong-password@localhost:5432/horbyta" -f database/schema.sql
```

**4. Create your environment file** (`/opt/horbyta/.env`) — see [Configuration reference](#configuration-reference) for every variable. A minimal file looks like:

```env
DATABASE_URL=postgresql://horbyta:change-me-strong-password@localhost:5432/horbyta
APP_URI=127.0.0.1:8890
APP_ENV=production
JWT_SECRET=replace-with-32-plus-random-characters
JWT_SECRET_CLIENT=replace-with-another-32-plus-random-string
TIME_TO_EXP_SESSION=7d
ALLOW_REGISTRATION=true
STATIC_DIR=/opt/horbyta/frontend/dist/pwa
UPLOAD_DIR=/opt/horbyta/data/storage
USER_STORAGE_LIMIT_GB=2
```

Generate strong secrets with:

```bash
openssl rand -hex 32
```

**5. Run it as a systemd service.** Create `/etc/systemd/system/horbyta.service`:

```ini
[Unit]
Description=Horbyta PM
After=network.target postgresql.service

[Service]
Type=simple
User=horbyta
WorkingDirectory=/opt/horbyta
EnvironmentFile=/opt/horbyta/.env
ExecStart=/opt/horbyta/horbyta
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Then:

```bash
sudo useradd -r -s /bin/false horbyta
sudo chown -R horbyta:horbyta /opt/horbyta
sudo systemctl daemon-reload
sudo systemctl enable --now horbyta
sudo systemctl status horbyta
```

You should see _"Server running at http://127.0.0.1:8890"_. Set `ALLOW_REGISTRATION=true` long enough to create your first user, then flip it back to `false`.

### Option B — Manual install from source

Use this path if you want to build it yourself, run an unreleased commit, or modify the code.

**Requirements:** Rust 1.88+, Node.js 22+, PostgreSQL 16+.

```bash
git clone https://github.com/HormigaDev/horbyta.git
cd horbyta

# 1. Build the frontend
cd frontend
npm ci
npx quasar build -m pwa
cd ..

# 2. Build the backend
cargo build --release

# 3. Prepare the database
psql "$DATABASE_URL" -f database/schema.sql

# 4. Configure environment
cp .env.template .env
# edit .env with your secrets and DATABASE_URL

# 5. Run
./target/release/horbyta
```

### Option C — Docker (maximum version safety)

Pick this path if your VPS ships outdated Rust or Node.js versions and you want the build to be fully reproducible regardless of the host. The included `Dockerfile` pins the exact toolchains used to build Horbyta (Rust 1.88, Node 22, Debian Bookworm runtime), so the only thing your VPS needs is Docker itself. PostgreSQL stays outside the container — point `DATABASE_URL` at any reachable database, managed or self-hosted.

[![Docker Compose](https://img.shields.io/badge/Compose-Multi--Stage%20Build-2496ED?logo=docker&logoColor=white)](#)
[![Reproducible Build](https://img.shields.io/badge/Build-Reproducible-success)](#)

```bash
git clone https://github.com/HormigaDev/horbyta.git
cd horbyta

# 1. Create your .env (uses the same template as Option B)
make setup
# edit .env with your DATABASE_URL, JWT secrets, etc.

# 2. Apply the schema against your PostgreSQL instance
psql "$DATABASE_URL" -f database/schema.sql

# 3. Build and start the container
make prod        # production: builds image and runs detached
# or, for development:
make dev         # rebuilds and starts the dev container
make logs        # tail backend logs
make down        # stop everything
```

The container exposes `127.0.0.1:${RUST_PORT:-8890}` on the host — put Nginx in front of it the same way you would for Option A. Attachments are mounted from `./data/storage` on the host, so backups and updates do not lose files.

---

## Configuration reference

All configuration is done via environment variables (or an `.env` file in the working directory).

| Variable                | Required | Default                 | Description                                              |
| ----------------------- | -------- | ----------------------- | -------------------------------------------------------- |
| `DATABASE_URL`          | yes      | —                       | PostgreSQL connection string                             |
| `APP_URI`               | no       | `127.0.0.1:8890`        | Address the server binds to                              |
| `APP_ENV`               | no       | `development`           | Set to `production` to disable permissive CORS           |
| `JWT_SECRET`            | yes      | —                       | Secret for user JWTs. 32+ random chars                   |
| `JWT_SECRET_CLIENT`     | yes      | —                       | Separate secret for client-portal JWTs                   |
| `TIME_TO_EXP_SESSION`   | yes      | —                       | Session lifetime, e.g. `7d`, `12h`, `30m`                |
| `STATIC_DIR`            | no       | `frontend/dist/pwa`     | Path to the compiled PWA                                 |
| `FRONTEND_ORIGIN`       | dev only | `http://localhost:9000` | Allowed CORS origin in development                       |
| `ALLOW_REGISTRATION`    | no       | `false`                 | Enable the public sign-up endpoint                       |
| `UPLOAD_DIR`            | no       | `./data/storage`        | Local path for attachments (used if S3 is unset)         |
| `USER_STORAGE_LIMIT_GB` | no       | `2`                     | Per-user storage cap for attachments                     |
| `S3_BUCKET`             | no       | —                       | Enables S3 storage when set together with the keys below |
| `S3_REGION`             | no       | —                       | S3 region                                                |
| `S3_ENDPOINT`           | no       | —                       | Custom S3 endpoint (MinIO, R2, B2…)                      |
| `S3_ACCESS_KEY`         | no       | —                       | S3 access key                                            |
| `S3_SECRET_KEY`         | no       | —                       | S3 secret key                                            |
| `RUST_LOG`              | no       | `info`                  | Log level (e.g. `info`, `debug`, `info,sqlx=warn`)       |

Set `ALLOW_REGISTRATION=false` if you are the only user — registration is the only way to create accounts.

---

## Putting it behind Nginx (HTTPS + rate limiting)

The repository ships with a production-ready Nginx vhost that adds HTTPS, security headers, request-size limits, and per-IP rate limiting (general + stricter on auth endpoints).

```bash
sudo cp nginx/http-directives.conf /etc/nginx/conf.d/pm-http-directives.conf
sudo cp nginx/pm-api.conf /etc/nginx/sites-available/pm-api
sudo ln -s /etc/nginx/sites-available/pm-api /etc/nginx/sites-enabled/pm-api

# Adjust server_name to your own domain inside pm-api.conf, then:
sudo nginx -t && sudo systemctl reload nginx

# Issue a TLS certificate
sudo certbot --nginx -d your.domain.tld
```

The provided config proxies to `127.0.0.1:8890` and forwards WebSocket upgrades correctly.

---

## First run

1. Open the app in your browser at the configured domain (or `http://127.0.0.1:8890` if you skipped Nginx).
2. Set `ALLOW_REGISTRATION=true`, restart, and create your account through the sign-up screen.
3. Set `ALLOW_REGISTRATION=false` again and restart, so no one else can sign up on your instance.
4. The first time a user is created, Horbyta seeds default categories (General, Work, Personal, Admin), default activities (Development, Design, Meeting, …), and default tags so you can start working immediately.

---

## Using Horbyta

Once you are logged in, the typical workflow looks like this:

1. **Add a client.** Clients live independently from projects so you can have multiple projects per client.
2. **Create a project** under a category, link it to the client, and configure billing (fixed / hourly / retainer).
3. **Break the project into milestones** with due dates so progress is visible.
4. **Create tasks** under the project (or under a milestone), nest subtasks as deeply as needed, tag them, and set priority.
5. **Track your time** as you work. Either start a live timer on a task or add manual entries by activity.
6. **Issue an invoice** when it is time to bill: select the time entries to consolidate, set the amount and currency, and either keep it as `manual` or attach an external provider reference.
7. **Register payments** against the invoice. Refunds are first-class — they reference the original charge and never overwrite it.
8. **Use the change-request engine** whenever a client asks for extra work. Capture the request, add your technical assessment with cost and schedule impact, and let the client formally approve or decline. No approval, no work.
9. **Invite the client** to the portal (magic link or email + password) so they can review milestones and respond to change requests on their own.
10. **Read your dashboards** to see monthly revenue, who owes you what, where your billable ratio is leaking, and which projects are bottlenecked.

The frontend is a PWA: install it from your browser to get a desktop-app feel.

---

## Backups

Two things to back up: the **PostgreSQL database** and the **attachments directory** (if you use local storage).

```bash
# Database
pg_dump "$DATABASE_URL" --format=custom --file=horbyta-$(date +%F).dump

# Attachments (if using local storage)
tar -czf horbyta-storage-$(date +%F).tar.gz /opt/horbyta/data/storage
```

If you use S3, your bucket is your backup — just make sure versioning is enabled.

---

## Updating

1. Stop the service: `sudo systemctl stop horbyta`.
2. Back up the database (see above).
3. Replace the `horbyta` binary and the `frontend/dist/pwa/` directory with the new release.
4. Apply any new migrations from `database/` against your database.
5. Start the service: `sudo systemctl start horbyta`.

---

## Troubleshooting

| Symptom                                                 | Likely cause                                                   | Fix                                                                   |
| ------------------------------------------------------- | -------------------------------------------------------------- | --------------------------------------------------------------------- |
| `CRITICAL: Could not connect to Postgres` on startup    | Wrong `DATABASE_URL`, DB not running, or user lacks permission | Verify the connection string with `psql` first                        |
| `function uuidv7() does not exist` when applying schema | PostgreSQL older than 16                                       | Upgrade PostgreSQL to 16+                                             |
| 401 / 403 right after login                             | `JWT_SECRET` changed since the token was issued                | Re-login; never rotate the secret without invalidating sessions       |
| 429 Too Many Requests                                   | Nginx rate limit hit                                           | Adjust `pm_api` / `pm_api_auth` zones in `nginx/http-directives.conf` |
| Attachments fail to upload                              | `UPLOAD_DIR` not writable, or S3 keys wrong                    | Check folder ownership, or your S3 credentials                        |
| `daily_limit_exceeded` when logging time                | Daily-hours ceiling enforced by DB trigger                     | Raise `max_daily_hours` in user preferences (max 14)                  |

---

## License

Horbyta is released under the [MIT License](LICENSE). You can self-host it, modify it, and run it for your own freelance practice or for your team — no strings attached.
