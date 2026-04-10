# BudgetTool

Self-hosted personal finance tool for two users. Uploads bank transaction files and runs reports via hledger.

## Stack

- **API**: Rust + Axum (`/api`) — binds port 3000
- **Frontend**: React + TypeScript + Vite (`/frontend`)
- **Infra**: Docker Compose + nginx (`/infra`)
- **Engine**: hledger CLI (invoked as subprocess from the API)
- **Database**: SQLite (not yet added)

## Development

- `cd api && cargo run` — run API locally
- `cd frontend && npm run dev` — run frontend dev server
- `docker compose up --build` — run full stack via Docker

## CI

GitHub Actions runs on push/PR to main:
- API: fmt, clippy, build, test
- Frontend: lint, build
