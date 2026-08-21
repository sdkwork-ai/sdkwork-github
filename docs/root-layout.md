# Root Layout

SDKWork GitHub uses the standard project-root dictionary.

| Directory | Purpose |
| --- | --- |
| `apis/` | OpenAPI authorities for app/backend/open surfaces |
| `apps/sdkwork-github-pc/` | PC React application surface |
| `crates/` | Rust API server, route crates, services, repositories |
| `database/` | `sdkwork-database` lifecycle assets |
| `sdks/` | Route manifests and SDK families |
| `specs/` | Component and topology contracts |
| `etc/topology/` | Runtime topology profiles |

Framework integration:

- `sdkwork-web-framework` via `sdkwork-web-axum` and `sdkwork-iam-web-adapter`
- `sdkwork-database` via `sdkwork-database-config` and migrations under `database/`
- `sdkwork-utils` via `sdkwork-utils-rust` and `@sdkwork/utils`
- `sdkwork-discovery` — not integrated (no RPC services yet)
