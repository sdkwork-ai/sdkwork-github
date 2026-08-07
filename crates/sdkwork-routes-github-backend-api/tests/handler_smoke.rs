use axum::body::to_bytes;
use axum::extract::{Query, State};
use axum::response::Response;
use http::StatusCode;
use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool};
use sdkwork_github_integration_repository_sqlx::SqlGitHubStore;
use sdkwork_github_integration_service::domain::LinkIntegrationCommand;
use sdkwork_github_integration_service::ports::GitHubSyncStore;
use sdkwork_github_integration_service::GitHubIntegrationService;
use sdkwork_routes_github_backend_api::dto::PageQuery;
use sdkwork_routes_github_backend_api::handlers;
use sdkwork_routes_github_backend_api::state::GitHubBackendState;
use sdkwork_web_core::{
    ServerRequestId, WebApiSurface, WebAuthMode, WebEnvironment, WebLoginScope, WebRequestContext,
    WebRequestPrincipal, WebTransportFacts,
};

async fn migrated_store() -> Option<SqlGitHubStore> {
    let Some(database_url) = optional_postgres_database_url() else {
        eprintln!("skipping handler smoke test: set SDKWORK_DATABASE_URL or DATABASE_URL to a postgres URL");
        return None;
    };
    let pool = create_pool_from_config(DatabaseConfig {
        engine: DatabaseEngine::Postgres,
        url: database_url,
        ..Default::default()
    })
    .await
    .expect("create postgres pool");
    Some(SqlGitHubStore::new(pool))
}


fn optional_postgres_database_url() -> Option<String> {
    std::env::var("SDKWORK_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|url| DatabaseEngine::from_url(url) == Some(DatabaseEngine::Postgres))
}

fn test_context() -> WebRequestContext {
    WebRequestContext {
        request_id: ServerRequestId("req-test".to_owned()),
        api_surface: WebApiSurface::BackendApi,
        auth_mode: WebAuthMode::DualToken,
        transport: WebTransportFacts {
            path: "/backend/v3/api/github/integrations".to_owned(),
            method: "GET".to_owned(),
            auth_token_present: true,
            access_token_present: true,
            api_key_present: false,
            ingress_token_present: false,
            oauth_bearer_present: false,
            agent_token_present: false,
        },
        principal: Some(
            WebRequestPrincipal::builder()
                .tenant_id("100001")
                .organization_id(Some("0".to_owned()))
                .login_scope(WebLoginScope::Organization)
                .user_id("admin-test")
                .session_id(Some("session-test".to_owned()))
                .app_id("sdkwork-github")
                .environment(WebEnvironment::Test)
                .build(),
        ),
        locale: None,
        client_kind: None,
        operation: None,
        trace_id: Some("trace-test".to_owned()),
        idempotency_key: None,
    }
}

async fn response_json(response: Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    serde_json::from_slice(&body).expect("parse response json")
}

#[tokio::test]
async fn list_integrations_returns_linked_accounts() {
    let Some(store) = migrated_store().await else {
        eprintln!("skipping handler smoke test: set SDKWORK_DATABASE_URL or DATABASE_URL to a postgres URL");
        return;
    };
    store
        .link_integration(
            "100001",
            "0",
            "github",
            &LinkIntegrationCommand {
                access_token: "ghp_test".to_string(),
                external_account_id: Some("12345".to_string()),
                scopes: Some("repo".to_string()),
            },
            "cipher-test",
        )
        .await
        .expect("seed integration");

    let service = GitHubIntegrationService::new(store);
    let state = GitHubBackendState::new(service);
    let response = handlers::list_integrations(
        State(state),
        test_context(),
        Query(PageQuery {
            page: Some(1),
            page_size: Some(20),
        }),
    )
    .await;
    let payload = response_json(response).await;

    assert_eq!(payload["code"], 0);
    assert_eq!(payload["data"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(payload["data"]["items"][0]["tenant_id"].as_str().unwrap(), "100001");
    assert_eq!(payload["data"]["items"][0]["organization_id"].as_str().unwrap(), "0");
}

#[tokio::test]
async fn sync_integration_requires_linked_provider() {
    let Some(store) = migrated_store().await else {
        eprintln!("skipping handler smoke test: set SDKWORK_DATABASE_URL or DATABASE_URL to a postgres URL");
        return;
    };
    let service = GitHubIntegrationService::new(store);
    let state = GitHubBackendState::new(service);
    let response = handlers::sync_integration_repositories(
        State(state),
        test_context(),
        axum::Json(sdkwork_routes_github_backend_api::dto::AdminSyncRequest {
            tenant_id: "100001".to_string(),
            organization_id: "0".to_string(),
        }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}
