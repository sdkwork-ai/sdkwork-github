//! API assembly for sdkwork-github.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM: exports beyond the canonical materializer template.

mod bootstrap;
mod catalog;
mod generated;
mod readiness;

pub use bootstrap::{assemble_api_router, ApiAssembly, ApiAssemblyContribution, assemble_api_router_from_env, web_module};
pub use sdkwork_routes_github_app_api::APP_HTTP_ROUTES;
pub use sdkwork_routes_github_backend_api::BACKEND_HTTP_ROUTES;

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
