use axum::routing::{delete, get, post};
use axum::Router;
use sdkwork_github_integration_service::ports::{GitHubSyncStore, TrackerStore};
use sdkwork_github_integration_service::GitHubIntegrationService;

use crate::handlers;
use crate::paths;
use crate::state::GitHubAppState;

pub fn build_router<S>(service: GitHubIntegrationService<S>) -> Router
where
    S: GitHubSyncStore + TrackerStore + Clone + Send + Sync + 'static,
{
    Router::new()
        .route(paths::REPOSITORIES, get(handlers::list_repositories::<S>))
        .route(
            paths::REPOSITORIES_SYNC,
            post(handlers::sync_repositories::<S>),
        )
        .route(paths::ISSUES, get(handlers::list_issues::<S>))
        .route(paths::ISSUES_SYNC, post(handlers::sync_issues::<S>))
        .route(paths::PLANS, get(handlers::list_plans::<S>))
        .route(
            paths::CATALOG_BOOTSTRAP,
            post(handlers::bootstrap_notable_catalog::<S>),
        )
        .route(paths::INTEGRATION, get(handlers::get_integration_status::<S>))
        .route(paths::INTEGRATION, post(handlers::link_integration::<S>))
        .route(paths::INTEGRATION, delete(handlers::unlink_integration::<S>))
        .route(
            paths::INTEGRATION_OAUTH_BEGIN,
            post(handlers::begin_oauth_integration::<S>),
        )
        .route(
            paths::INTEGRATION_OAUTH_CALLBACK,
            get(handlers::oauth_callback::<S>),
        )
        // Tracker routes
        .route(paths::TRACKER_ISSUES, get(handlers::list_tracker_issues::<S>).post(handlers::create_tracker_issue::<S>))
        .route(paths::TRACKER_ISSUE_DETAIL, get(handlers::get_tracker_issue_detail::<S>).patch(handlers::update_tracker_issue::<S>))
        .route(paths::TRACKER_ISSUE_COMMENTS, get(handlers::list_tracker_comments::<S>).post(handlers::create_tracker_comment::<S>))
        .route(paths::TRACKER_ISSUE_VOTES, post(handlers::toggle_tracker_vote::<S>))
        .route(paths::TRACKER_ISSUE_VOTE_STATUS, get(handlers::get_tracker_vote_status::<S>))
        .route(paths::TRACKER_LABELS, get(handlers::list_tracker_labels::<S>).post(handlers::create_tracker_label::<S>))
        .route(paths::TRACKER_MILESTONES, get(handlers::list_tracker_milestones::<S>).post(handlers::create_tracker_milestone::<S>))
        .route(paths::TRACKER_MILESTONE_ISSUES, get(handlers::get_tracker_milestone_issues::<S>))
        .route(paths::TRACKER_ROADMAPS, get(handlers::list_tracker_roadmaps::<S>).post(handlers::create_tracker_roadmap::<S>))
        .route(paths::TRACKER_ROADMAP_DETAIL, get(handlers::get_tracker_roadmap_detail::<S>).patch(handlers::update_tracker_roadmap::<S>))
        .route(paths::TRACKER_ROADMAP_ITEMS, post(handlers::add_tracker_roadmap_item::<S>))
        .route(paths::TRACKER_ROADMAP_ITEM_DETAIL, delete(handlers::remove_tracker_roadmap_item::<S>))
        .with_state(GitHubAppState::new(service))
}
