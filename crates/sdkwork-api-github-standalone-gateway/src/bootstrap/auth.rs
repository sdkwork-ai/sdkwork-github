use axum::Router;

use crate::web_bootstrap::wrap_router_with_web_framework_from_env;

pub async fn build_protected_router(router: Router) -> Router {
    wrap_router_with_web_framework_from_env(router).await
}
