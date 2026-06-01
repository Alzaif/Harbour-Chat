# Harbour Chat — agent notes

Hexagonal satellite: **Rust** in `api/`, **React** in `web/`.

## Layers (`api/src/`)

| Layer | Path | Rules |
|-------|------|--------|
| Domain | `domain/` | Entities, port traits, events — no axum/sqlx |
| Application | `application/` | `ChatService` orchestration |
| Infrastructure | `infrastructure/` | Axum, sqlx, WebSocket hub, filesystem |
| Contracts | `contracts/` | Gateway headers, integration event shapes |

Presentation (`web/`) calls HTTP/WebSocket only — never imports Rust modules.

## Commands

- `cd api && cargo test`
- `cd web && npm run typecheck && npm test`
- Dev: API `3004`, Vite `5177`

## Platform

Register in `portcullis/config/harbour-apps.json` and `harbour-infra` compose like other satellites.
