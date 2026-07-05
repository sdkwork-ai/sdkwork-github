# SDKWork GitHub Technical Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-02
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- [TECH-root-layout.md](TECH-root-layout.md)
- [TECH-standard-alignment-audit.md](TECH-standard-alignment-audit.md)

## 1. Architecture Overview

SDKWork GitHub is a monorepo application with a Rust backend (axum) and React PC frontend. The backend exposes an `app-api` surface at `/app/v3/api/github/*` with standard SDKWork response envelopes. The frontend is organized into domain packages under `apps/sdkwork-github-pc/packages/`.

Architecture detail lives in the linked TECH shards below.

## 2. Technology Choices

- **Backend**: Rust, axum, sqlx (SQLite + PostgreSQL), async-trait.
- **Frontend**: React 19, react-router-dom 7, Vite 6, TypeScript 5.
- **Database**: SQLite (development) and PostgreSQL (production) via `sdkwork-database-sqlx`.
- **API**: OpenAPI 3.0 specification with SDKWork response envelope (`SdkWorkApiResponse`).
- **SDK**: Generated TypeScript SDK with `data` unwrapping (`--standard-profile sdkwork-v3`).

## 3. System Boundaries And Modules

### Backend Crates

- `sdkwork-github-integration-service`: Domain types, service layer, port traits (`GitHubStore`, `GitHubSyncStore`, `TrackerStore`).
- `sdkwork-github-integration-repository-sqlx`: SQLx implementation of store traits.
- `sdkwork-github-integration-provider-github`: GitHub REST API and OAuth client.
- `sdkwork-routes-github-app-api`: Axum route handlers, DTOs, route manifest.
- `sdkwork-routes-github-common`: Shared response helpers (`ApiProblem`, `finish_api_json`, `item_data`, `list_page_data`).
- `sdkwork-github-gateway-assembly`: Gateway bootstrap and router assembly.
- `sdkwork-github-standalone-gateway`: Standalone binary entry point.

### Frontend Packages

- `sdkwork-github-pc-core`: Runtime, SDK client, session management.
- `sdkwork-github-pc-shell`: Application shell, routing, layout.
- `sdkwork-github-pc-workspace`: Workspace navigation.
- `sdkwork-github-pc-tracker`: Tracker domain (Issues, Roadmaps, Milestones).

## 4. Directory And Package Layout

```
sdkwork-github/
  crates/
    sdkwork-github-integration-service/       # Domain + service + ports
    sdkwork-github-integration-repository-sqlx/ # SQLx store impl
    sdkwork-github-integration-provider-github/ # GitHub provider
    sdkwork-routes-github-app-api/            # API handlers + routes
    sdkwork-routes-github-common/             # Shared response helpers
    sdkwork-github-gateway-assembly/          # Gateway bootstrap
    sdkwork-github-standalone-gateway/        # Standalone binary
  apps/sdkwork-github-pc/
    packages/
      sdkwork-github-pc-core/                 # Runtime + SDK client
      sdkwork-github-pc-shell/                # App shell + routing
      sdkwork-github-pc-workspace/            # Workspace nav
      sdkwork-github-pc-tracker/              # Tracker domain UI
  apis/app-api/github/                        # OpenAPI spec
  database/ddl/baseline/                      # SQL DDL (sqlite + postgres)
  sdks/sdkwork-github-app-sdk/                # Generated TypeScript SDK
```

## 5. API, SDK, And Data Ownership

### API Surface

- Base path: `/app/v3/api/github/`
- Tracker namespace: `/app/v3/api/github/tracker/`
- Response envelope: `SdkWorkApiResponse` with `{ code: 0, data, traceId }`
- Single resource: `data.item`
- Lists: `data.items` + `data.pageInfo`
- Commands: `data.accepted`

### Tracker Endpoints

| Method | Path | Description |
| --- | --- | --- |
| GET | `/tracker/issues` | List issues with filters |
| POST | `/tracker/issues` | Create issue |
| GET | `/tracker/issues/{id}` | Get issue detail (with labels + milestone) |
| PATCH | `/tracker/issues/{id}` | Update issue |
| GET | `/tracker/issues/{id}/comments` | List comments |
| POST | `/tracker/issues/{id}/comments` | Create comment |
| POST | `/tracker/issues/{id}/votes` | Toggle vote |
| GET | `/tracker/issues/{id}/votes/status` | Get vote status |
| GET | `/tracker/labels` | List labels |
| POST | `/tracker/labels` | Create label |
| GET | `/tracker/milestones` | List milestones with progress |
| POST | `/tracker/milestones` | Create milestone |
| GET | `/tracker/milestones/{id}/issues` | List issues in milestone |
| GET | `/tracker/roadmaps` | List roadmaps |
| POST | `/tracker/roadmaps` | Create roadmap |
| GET | `/tracker/roadmaps/{id}` | Get roadmap detail (with items) |
| PATCH | `/tracker/roadmaps/{id}` | Update roadmap |
| POST | `/tracker/roadmaps/{id}/items` | Add issue to roadmap |
| DELETE | `/tracker/roadmaps/{id}/items/{itemId}` | Remove roadmap item |

### Data Ownership

- Tracker data is tenant-scoped (`tenant_id` + `organization_id`).
- `github_tracker_issue`: Core issue records.
- `github_tracker_label`: Label catalog.
- `github_tracker_issue_label`: Issue-label join.
- `github_tracker_milestone`: Milestone definitions.
- `github_tracker_comment`: Issue comments.
- `github_tracker_vote`: User votes (unique per user+issue).
- `github_tracker_roadmap`: Roadmap definitions.
- `github_tracker_roadmap_item`: Issues placed on a roadmap.

## 6. Security, Privacy, And Observability

- OAuth state tokens expire after 10 minutes.
- GitHub access tokens are encrypted at rest via `GitHubCredentialCipher`.
- All API endpoints require authenticated principal (`WebRequestContext`).
- Tenant scope validation (`validate_scope`) on every service method.
- Tracing via `tracing` crate for catalog sync and provider operations.

## 7. Deployment And Runtime Topology

- Development: `pnpm dev` starts browser dev server with SQLite.
- Build: `pnpm build` compiles Rust workspace.
- Standalone gateway: `sdkwork-github-standalone-gateway` binary.
- Database lifecycle: `pnpm db:*` commands via `sdkwork-database` CLI.

## 8. Architecture Decision Index

- **ADR-001**: Single consolidated tracker package (`sdkwork-github-pc-tracker`) for domain cohesion.
- **ADR-002**: `/tracker/` namespace to avoid path conflicts with GitHub sync endpoints.
- **ADR-003**: Cross-database compatibility via conditional placeholder function (`ph()`).
- **ADR-004**: `TrackerStore` trait extends `GitHubStore` for dependency inversion.

## 9. Verification

- `pnpm check`: App composition, cargo check, pnpm script standard, architecture alignment, topology, database, API materialize, SDK generate, typecheck.
- `pnpm test`: Topology validation, contract tests, database framework tests, IAM bootstrap standard.
- `pnpm verify`: `cargo test --workspace` + `pnpm test`.
- API envelope check: `node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .`
