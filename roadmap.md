# BudgetTool Roadmap

A self-hosted personal finance tool for uploading bank transaction files and running reports.
Designed for two users (you and your partner).

## Architecture

| Layer | Technology |
|---|---|
| Engine | hledger CLI — invoked via subprocess, `--output-format=json` |
| API | Rust + Axum — auth, file storage, report proxying |
| Frontend | React + TypeScript (Vite) |
| Database | SQLite — user accounts, sessions, file metadata |
| Deployment | Docker Compose — API container + frontend via nginx |

---

## Phase 0 — Project Foundation

- Monorepo layout: `/api` (Rust), `/frontend` (React), `/infra` (Docker/nginx)
- Docker Compose dev environment with hledger available in-container
- Basic CI: lint and build checks

## Phase 1 — Core API

- Axum skeleton with health endpoint
- hledger CLI integration: subprocess invocation, JSON output parsing
- Shared error handling and response types

## Phase 2 — Authentication

- JWT-based auth (login/logout)
- SQLite user table (2 users)
- Protected route middleware

## Phase 3 — File Management

- Upload `.journal` and `.csv` files per user
- File listing and deletion endpoints
- CSV → hledger journal conversion via rules files

## Phase 4 — Reports API

Endpoints wrapping hledger commands:

- `balance` — account balances
- `incomestatement` — income vs. expenses
- `register` — transaction register
- `cashflow` — cash flow statement

All endpoints support date range and account filtering parameters.

## Phase 5 — Frontend Foundation

- React + Vite + TypeScript scaffold
- Client-side routing
- Auth flow (login page, protected routes)
- API client layer (typed fetch wrappers)

## Phase 6 — Frontend Reports & Dashboard

- Dashboard with summary widgets (net worth, monthly spend, etc.)
- Balance sheet, income/expense, and transaction register views
- File upload UI
- Date range pickers
- Charts (spending trends, category breakdown)

## Phase 7 — Deployment

- Production Docker Compose with nginx reverse proxy
- SSL via Let's Encrypt (Certbot)
- Deployment documentation

## Phase 8 — Future Enhancements

- Budget targets and variance tracking
- Recurring transaction templates
- Multi-journal merging (combine both partners' files into unified reports)
- Mobile-responsive polish
