//! API assembly for sdkwork-github.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.

mod bootstrap;
mod generated;

pub use bootstrap::{assemble_api_router, ApiAssembly};
pub use sdkwork_routes_github_app_api::APP_HTTP_ROUTES;
pub use sdkwork_routes_github_backend_api::BACKEND_HTTP_ROUTES;

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
