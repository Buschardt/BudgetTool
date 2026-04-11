# BudgetTool

A self-hosted personal finance tool for two users. Upload bank transaction files (hledger journals or CSVs), then run balance, income, cashflow, and register reports through a web UI.

## Architecture

```
browser → nginx (:8080)
               ├── /api/*  → Rust/Axum API (:3000)
               │               ├── auth (JWT)
               │               ├── file storage (journals on disk, metadata in SQLite)
               │               └── reports (invokes hledger as a subprocess)
               └── /*      → React frontend (served by nginx from built assets)
```

| Layer     | Technology                         |
|-----------|------------------------------------|
| Engine    | hledger CLI — subprocess, JSON output |
| API       | Rust + Axum                        |
| Frontend  | React + TypeScript + Vite          |
| Database  | SQLite (users, file metadata)      |
| Infra     | Docker Compose + nginx             |

## Repository Layout

```
/
├── api/                   Rust API (Axum)
│   ├── src/
│   │   ├── main.rs        Entry point; CLI with `serve` and `seed` subcommands
│   │   ├── lib.rs         Router definition — all routes wired here
│   │   ├── auth.rs        JWT login handler and extractor
│   │   ├── db.rs          SQLite pool init and migrations
│   │   ├── models.rs      AppState, Claims, and shared data types
│   │   ├── files.rs       Upload, list, delete, CSV-convert endpoints
│   │   ├── hledger.rs     subprocess wrapper — runs hledger and parses JSON
│   │   ├── reports.rs     balance, incomestatement, register, cashflow endpoints
│   │   ├── response.rs    Typed ApiResponse<T> envelope
│   │   └── error.rs       AppError and IntoResponse impl
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

| Method | Path                          | Description                        |
|--------|-------------------------------|------------------------------------|
| GET    | `/api/health`                 | Health check                       |
| POST   | `/api/login`                  | Obtain JWT                         |
| GET    | `/api/me`                     | Current user info (auth required)  |
| GET    | `/api/files`                  | List uploaded files                |
| POST   | `/api/files`                  | Upload a journal or CSV file       |
| GET    | `/api/files/:id`              | Download a file                    |
| DELETE | `/api/files/:id`              | Delete a file                      |
| POST   | `/api/files/:id/convert`      | Convert CSV to hledger journal     |
| GET    | `/api/reports/balance`        | Balance report                     |
| GET    | `/api/reports/incomestatement`| Income statement                   |
| GET    | `/api/reports/register`       | Transaction register               |
| GET    | `/api/reports/cashflow`       | Cash flow statement                |

All `/api/files` and `/api/reports` routes require a `Bearer` JWT in the `Authorization` header.

## Environment Variables

| Variable       | Description                                 | Default (dev)              |
|----------------|---------------------------------------------|----------------------------|
| `DATABASE_URL` | SQLite connection string                    | `sqlite:data/budgettool.db?mode=rwc` |
| `JWT_SECRET`   | Secret used to sign/verify JWTs            | *(required)*               |
| `DATA_DIR`     | Directory where uploaded files are stored   | `data/files`               |

## CI

GitHub Actions runs on every push and pull request to `main`:

- **API**: `cargo fmt --check`, `cargo clippy`, `cargo build`, `cargo test`
- **Frontend**: `eslint`, `vite build`
