# SDKWork GitHub 标准对齐审计

最后更新：2026-06-29

## 总体结论

| 维度 | 状态 | 说明 |
| --- | --- | --- |
| sdkwork-specs 字典与目录结构 | 已对齐 | 标准根目录、`AGENTS.md`、`.sdkwork/`、`specs/` |
| sdkwork-web-framework | 已对齐 | `WebFrameworkLayer` + `HttpRouteManifest` + `service_router` 健康探针 |
| sdkwork-database | 已对齐 | `database.manifest.json`、migrations、seed-on-boot 拓扑开关 |
| sdkwork-utils | 已对齐 | Rust `sdkwork-utils-rust` + PC `@sdkwork/utils` |
| sdkwork-discovery | 不适用 | 当前无 gRPC/RPC 服务 |
| 部署与打包 | 已对齐 | `sdkwork.workflow.json` + 薄 GitHub workflow |
| 生产就绪 | 已对齐 | `health.rs` 就绪探针与指标快照、OAuth/catalog 拓扑配置 |

## 近期对齐项（2026-06-29）

- Route manifest 文件名统一为 `sdkwork-routes-github-*.route-manifest.json`
- 新增 `sdkwork-routes-github-common`，handlers 经 `SdkWorkApiResponse` + `ProblemDetail` 输出
- `standalone-gateway` 新增 `health.rs`（`ready_check` + `metrics_snapshot`），经 `service_router` 挂载探针
- 开发拓扑启用 `SDKWORK_DATABASE_SEED_ON_BOOT` 与 `SDKWORK_GITHUB_CATALOG_SYNC_ON_BOOT`；生产拓扑关闭 catalog 启动同步
- PC 依赖 workspace 统一在仓库根 `pnpm-workspace.yaml`（禁止 app 级嵌套 workspace）；composition 契约指向 `specs/component.spec.json`

## 验证命令

```bash
pnpm check
pnpm verify
node --test scripts/verify-github-standard-architecture.test.mjs
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
```
