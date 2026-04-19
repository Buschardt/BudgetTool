# BudgetTool

A self-hosted personal finance tool for two users. Upload bank transaction files (hledger journals or CSVs), then run balance, income, cashflow, and register reports through a web UI.

## Architecture

```
browser → nginx (:8080)
               ├── /api/*  → Rust/Axum API (:3000)
               │               ├── auth (JWT)
               │               ├── file storage (journals on disk, metadata in SQLite)
               │               ├── rules configs (.rules file generation)
               │               ├── manual entries (prices, transactions, periodics)
               │               └── reports (invokes hledger as a subprocess)
               └── /*      → React frontend (served by nginx from built assets)
```

| Layer     | Technology                            |
|-----------|---------------------------------------|
| Engine    | hledger CLI — subprocess, JSON output |
| API       | Rust + Axum                           |
| Frontend  | React + TypeScript + Vite             |
| Database  | SQLite (users, file metadata)         |
| Infra     | Docker Compose + nginx                |

## Repository Layout

```
/
├── api/                   Rust API (Axum)
│   ├── src/
│   │   ├── main.rs        Entry point; CLI with `serve` and `seed` subcommands
│   │   ├── lib.rs         Module declarations and pub use re-exports
│   │   ├── routes.rs      Composes all feature routers into the root Router
│   │   │
│   │   ├── core/          Cross-cutting infrastructure
│   │   │   ├── state.rs   AppState (db pool, jwt secret, data dir)
│   │   │   ├── error.rs   AppError and IntoResponse impl
│   │   │   ├── response.rs Typed ApiResponse<T> envelope
│   │   │   ├── db.rs      SQLite pool init and migrations
│   │   │   └── hledger.rs subprocess wrapper — runs hledger, parses JSON
│   │   │
│   │   ├── auth/          Login and JWT extraction
│   │   │   ├── handlers.rs login, me
│   │   │   ├── extractor.rs FromRequestParts impl for Claims
│   │   │   ├── jwt.rs     encode_jwt / decode_jwt
│   │   │   └── models.rs  User, Claims, LoginRequest, LoginResponse
│   │   │
│   │   ├── files/         File upload and CSV conversion
│   │   │   ├── handlers.rs upload, list, get_one, delete, convert_csv, create_journal
│   │   │   ├── models.rs  FileRecord, FileInfo
│   │   │   └── filename.rs sanitize_filename, normalize_journal_name, file_extension
│   │   │
│   │   ├── rules/         Rules config CRUD and .rules file generation
│   │   │   ├── handlers.rs list, get_one, create, update, delete, preview
│   │   │   ├── service.rs  resolve_include_paths, generate_and_write, resolve_rules_path
│   │   │   ├── models.rs  RulesConfigRecord, RulesConfigInfo, RulesConfigDetail
│   │   │   └── generator.rs .rules text generation and validation
│   │   │
│   │   ├── manual_entries/ Manually entered journal data
│   │   │   ├── prices.rs  Commodity price CRUD handlers
│   │   │   ├── transactions.rs Manual transaction CRUD handlers
│   │   │   ├── periodics.rs Periodic transaction CRUD handlers
│   │   │   ├── journal.rs  regenerate_journal (writes manual-entries.journal)
│   │   │   ├── models.rs  Record and Info types for all three entity kinds
│   │   │   └── generator.rs hledger journal text generation
│   │   │
│   │   └── reports/       hledger report endpoints
│   │       ├── handlers.rs balance, income_statement, register, cashflow
│   │       └── journals.rs journal_args, filter_args, build_args helpers
│   │
│   ├── migrations/        SQL migration files (run on startup)
│   ├── data/              Local dev data directory (DB + uploaded files)
│   ├── tests/             Integration tests (auth, hledger)
│   └── .env.example       Required environment variables
│
├── frontend/              React + TypeScript (Vite)
│   └── src/
│       ├── main.tsx        App entry; routing setup
│       ├── auth.tsx        Auth context and useAuth hook
│       ├── api.ts          Typed fetch wrappers for every API endpoint
│       ├── types/          hledger response types
│       ├── lib/            Utilities (hledger JSON parsing, number formatting)
│       ├── hooks/          useReport — shared data fetching hook
│       ├── components/     Reusable UI (Layout, DataTable, FileUploader, etc.)
│       └── pages/          One file per route:
│                           LoginPage, RegisterPage, DashboardPage,
│                           BalancePage, IncomePage, CashflowPage, FilesPage
│
├── infra/
│   ├── Dockerfile.api      Multi-stage Rust build
│   ├── Dockerfile.frontend Vite build → nginx static serve
│   └── nginx.conf          Reverse proxy: /api/* → API, /* → frontend
│
├── docker-compose.yml      Full-stack dev/prod compose file
└── .github/workflows/ci.yml  CI: API (fmt, clippy, build, test) + Frontend (lint, build)
```

## Development

### Prerequisites

- Rust toolchain (`rustup`)
- Node.js 20+
- hledger installed and on `$PATH`

### Run locally

**API**

```bash
cp api/.env.example api/.env   # fill in JWT_SECRET
cd api && cargo run
# API available at http://localhost:3000
```

**Frontend**

```bash
cd frontend && npm install && npm run dev
# UI available at http://localhost:5173
```

### Run with Docker

```bash
JWT_SECRET=your-secret docker compose up --build
# Full stack at http://localhost:8080
```

### Seed a user

The API ships no registration endpoint — users are seeded directly:

```bash
cd api && cargo run -- seed --username alice --password hunter2
```

## API Endpoints

All endpoints under `/api/files`, `/api/reports`, `/api/rules-configs`, `/api/prices`, `/api/transactions`, and `/api/periodics` require a `Bearer` JWT in the `Authorization` header.

| Method | Path                              | Description                              |
|--------|-----------------------------------|------------------------------------------|
| GET    | `/api/health`                     | Health check                             |
| POST   | `/api/login`                      | Obtain JWT                               |
| GET    | `/api/me`                         | Current user info                        |
| GET    | `/api/files`                      | List uploaded files                      |
| POST   | `/api/files`                      | Upload a journal, CSV, or rules file     |
| GET    | `/api/files/{id}`                 | Get file metadata                        |
| DELETE | `/api/files/{id}`                 | Delete a file                            |
| POST   | `/api/files/{id}/convert`         | Convert CSV to hledger journal           |
| POST   | `/api/journals`                   | Create an empty named journal            |
| GET    | `/api/rules-configs`              | List rules configs                       |
| POST   | `/api/rules-configs`              | Create a rules config                    |
| GET    | `/api/rules-configs/{id}`         | Get a rules config                       |
| PUT    | `/api/rules-configs/{id}`         | Update a rules config                    |
| DELETE | `/api/rules-configs/{id}`         | Delete a rules config                    |
| POST   | `/api/rules-configs/{id}/preview` | Preview rules config against a CSV       |
| GET    | `/api/prices`                     | List commodity prices                    |
| POST   | `/api/prices`                     | Create a commodity price                 |
| PUT    | `/api/prices/{id}`                | Update a commodity price                 |
| DELETE | `/api/prices/{id}`                | Delete a commodity price                 |
| GET    | `/api/transactions`               | List manual transactions                 |
| POST   | `/api/transactions`               | Create a manual transaction              |
| PUT    | `/api/transactions/{id}`          | Update a manual transaction              |
| DELETE | `/api/transactions/{id}`          | Delete a manual transaction              |
| GET    | `/api/periodics`                  | List periodic transactions               |
| POST   | `/api/periodics`                  | Create a periodic transaction            |
| PUT    | `/api/periodics/{id}`             | Update a periodic transaction            |
| DELETE | `/api/periodics/{id}`             | Delete a periodic transaction            |
| GET    | `/api/reports/balance`            | Balance report                           |
| GET    | `/api/reports/incomestatement`    | Income statement                         |
| GET    | `/api/reports/register`           | Transaction register                     |
| GET    | `/api/reports/cashflow`           | Cash flow statement                      |

Report endpoints accept optional query parameters: `begin`, `end`, `period`, `depth`, `account`.

## Environment Variables

| Variable       | Description                                 | Default (dev)                        |
|----------------|---------------------------------------------|--------------------------------------|
| `DATABASE_URL` | SQLite connection string                    | `sqlite:data/budgettool.db?mode=rwc` |
| `JWT_SECRET`   | Secret used to sign/verify JWTs            | *(required)*                         |
| `DATA_DIR`     | Directory where uploaded files are stored   | `data/files`                         |

## CI

GitHub Actions runs on every push and pull request to `main`:

- **API**: `cargo fmt --check`, `cargo clippy`, `cargo build`, `cargo test`
- **Frontend**: `eslint`, `vite build`
