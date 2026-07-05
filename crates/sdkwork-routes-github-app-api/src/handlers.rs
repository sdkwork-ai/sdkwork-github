use axum::extract::{Path, Query, State};
use axum::response::{Redirect, Response};
use axum::Json;
use http::StatusCode;
use sdkwork_github_integration_service::domain::{
    CreateTrackerIssueCommand, CreateTrackerRoadmapItemCommand, TrackerIssueQuery,
    UpdateTrackerIssueCommand,
};
use sdkwork_github_integration_service::ports::{GitHubSyncStore, TrackerStore};
use sdkwork_routes_github_common::{
    finish_api_json, item_data, list_page_data, map_service_error, ApiProblem, ApiResult,
};
use sdkwork_utils_rust::string::is_blank;
use sdkwork_web_core::WebRequestContext;

use crate::dto::{
    CatalogBootstrapResponse, CommandAcceptedResponse, CreateTrackerCommentRequest,
    CreateTrackerIssueRequest, CreateTrackerLabelRequest, CreateTrackerMilestoneRequest,
    CreateTrackerRoadmapItemRequest, CreateTrackerRoadmapRequest, IntegrationStatusResponse,
    LinkIntegrationRequest, MilestoneIssueListQuery, OAuthBeginResponse, OAuthCallbackQuery,
    PageQuery, SyncResponse, TrackerIssueListQuery, TrackerLabelsResponse,
    TrackerMilestonesResponse, UpdateTrackerIssueRequest, UpdateTrackerRoadmapRequest,
    VoteResponse, VoteStatusResponse,
};
use crate::state::GitHubAppState;

fn resolve_scope(
    app_ctx: &WebRequestContext,
    query: &PageQuery,
) -> Result<(String, String), ApiProblem> {
    let principal = app_ctx
        .principal
        .as_ref()
        .ok_or_else(|| ApiProblem::unauthorized("authenticated principal is required"))?;
    let tenant_id = query
        .tenant_id
        .clone()
        .filter(|value| !is_blank(Some(value.as_str())))
        .unwrap_or_else(|| principal.tenancy.tenant_id.clone());
    let organization_id = query
        .organization_id
        .clone()
        .filter(|value| !is_blank(Some(value.as_str())))
        .or_else(|| principal.tenancy.organization_id.clone())
        .ok_or_else(|| ApiProblem::bad_request("organization_id is required"))?;
    Ok((tenant_id, organization_id))
}

fn resolve_scope_with_user(
    app_ctx: &WebRequestContext,
    tenant_id: Option<&str>,
    organization_id: Option<&str>,
) -> Result<(String, String, String), ApiProblem> {
    let principal = app_ctx
        .principal
        .as_ref()
        .ok_or_else(|| ApiProblem::unauthorized("authenticated principal is required"))?;
    let tid = tenant_id
        .filter(|v| !is_blank(Some(v)))
        .map(|s| s.to_string())
        .unwrap_or_else(|| principal.tenancy.tenant_id.clone());
    let oid = organization_id
        .filter(|v| !is_blank(Some(v)))
        .map(|s| s.to_string())
        .or_else(|| principal.tenancy.organization_id.clone())
        .ok_or_else(|| ApiProblem::bad_request("organization_id is required"))?;
    let uid = principal.subject.user_id.clone();
    Ok((tid, oid, uid))
}

pub async fn list_repositories<S: GitHubSyncStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
) -> Response {
    let result = (|| async {
        let (tenant_id, organization_id) = resolve_scope(&app_ctx, &query)?;
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let page = state
            .service
            .list_repositories(&tenant_id, &organization_id, page, page_size)
            .await
            .map_err(map_service_error)?;
        Ok(list_page_data(page.items, page.page, page.page_size, page.total))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn sync_repositories<S: GitHubSyncStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
) -> Response {
    let result = (|| async {
        let (tenant_id, organization_id) = resolve_scope(&app_ctx, &query)?;
        let sync = state
            .service
            .sync_repositories(&tenant_id, &organization_id)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(SyncResponse::from(sync)))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn list_issues<S: GitHubSyncStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
) -> Response {
    let result = (|| async {
        let (tenant_id, organization_id) = resolve_scope(&app_ctx, &query)?;
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let repository_id = query.repository_id.as_deref();
        let page = state
            .service
            .list_issues(
                &tenant_id,
                &organization_id,
                repository_id,
                page,
                page_size,
            )
            .await
            .map_err(map_service_error)?;
        Ok(list_page_data(page.items, page.page, page.page_size, page.total))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn sync_issues<S: GitHubSyncStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
) -> Response {
    let result = (|| async {
        let (tenant_id, organization_id) = resolve_scope(&app_ctx, &query)?;
        let repository_id = query.repository_id.as_deref();
        let sync = state
            .service
            .sync_issues(&tenant_id, &organization_id, repository_id)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(SyncResponse::from(sync)))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn list_plans<S: GitHubSyncStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
) -> Response {
    let result = (|| async {
        let (tenant_id, organization_id) = resolve_scope(&app_ctx, &query)?;
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let page = state
            .service
            .list_plans(&tenant_id, &organization_id, page, page_size)
            .await
            .map_err(map_service_error)?;
        Ok(list_page_data(page.items, page.page, page.page_size, page.total))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn get_integration_status<S: GitHubSyncStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
) -> Response {
    let result = (|| async {
        let (tenant_id, organization_id) = resolve_scope(&app_ctx, &query)?;
        let status = state
            .service
            .get_integration_status(&tenant_id, &organization_id)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(IntegrationStatusResponse::from(status)))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn link_integration<S: GitHubSyncStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
    Json(body): Json<LinkIntegrationRequest>,
) -> Response {
    let result = (|| async {
        let (tenant_id, organization_id) = resolve_scope(&app_ctx, &query)?;
        let status = state
            .service
            .link_integration(&tenant_id, &organization_id, body.into())
            .await
            .map_err(map_service_error)?;
        Ok(item_data(IntegrationStatusResponse::from(status)))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn unlink_integration<S: GitHubSyncStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
) -> Response {
    let result = (|| async {
        let (tenant_id, organization_id) = resolve_scope(&app_ctx, &query)?;
        let status = state
            .service
            .unlink_integration(&tenant_id, &organization_id)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(IntegrationStatusResponse::from(status)))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn bootstrap_notable_catalog<S: GitHubSyncStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
) -> Response {
    let result = (|| async {
        let (tenant_id, organization_id) = resolve_scope(&app_ctx, &query)?;
        let bootstrap = state
            .service
            .bootstrap_notable_catalog(&tenant_id, &organization_id)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(CatalogBootstrapResponse::from(bootstrap)))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn begin_oauth_integration<S: GitHubSyncStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
) -> Response {
    let result = (|| async {
        let (tenant_id, organization_id) = resolve_scope(&app_ctx, &query)?;
        let oauth = state
            .service
            .begin_oauth_integration(&tenant_id, &organization_id)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(OAuthBeginResponse::from(oauth)))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn oauth_callback<S: GitHubSyncStore>(
    State(state): State<GitHubAppState<S>>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<Redirect, (StatusCode, String)> {
    let success_redirect = std::env::var("SDKWORK_GITHUB_OAUTH_SUCCESS_REDIRECT_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:5175/integration".to_string());
    match state
        .service
        .complete_oauth_integration(&query.state, &query.code)
        .await
    {
        Ok(_) => Ok(Redirect::temporary(&format!("{success_redirect}?linked=1"))),
        Err(error) => {
            let error_message = error.to_string();
            let message = urlencoding::encode(&error_message);
            Ok(Redirect::temporary(&format!(
                "{success_redirect}?linked=0&error={message}"
            )))
        }
    }
}

// ════════════════════════════════════════════════════════
// Tracker handlers
// ════════════════════════════════════════════════════════

pub async fn list_tracker_issues<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<TrackerIssueListQuery>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (tenant_id, organization_id, _) =
            resolve_scope_with_user(&app_ctx, query.tenant_id.as_deref(), query.organization_id.as_deref())?;
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let domain_query = TrackerIssueQuery {
            issue_type: query.issue_type,
            status: query.status,
            priority: query.priority,
            milestone_id: query.milestone_id,
            label_id: query.label_id,
            q: query.q,
            sort: query.sort,
        };
        let page_result = state
            .service
            .list_tracker_issues(&tenant_id, &organization_id, &domain_query, page, page_size)
            .await
            .map_err(map_service_error)?;
        Ok(list_page_data(page_result.items, page_result.page, page_result.page_size, page_result.total))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn get_tracker_issue_detail<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
    Path(issue_id): Path<String>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (tenant_id, organization_id, _) =
            resolve_scope_with_user(&app_ctx, query.tenant_id.as_deref(), query.organization_id.as_deref())?;
        let view = state
            .service
            .get_tracker_issue_detail(&tenant_id, &organization_id, &issue_id)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(view))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn create_tracker_issue<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
    Json(body): Json<CreateTrackerIssueRequest>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (tenant_id, organization_id, user_id) =
            resolve_scope_with_user(&app_ctx, query.tenant_id.as_deref(), query.organization_id.as_deref())?;
        let command = CreateTrackerIssueCommand {
            title: body.title,
            description: body.description,
            issue_type: body.issue_type,
            priority: body.priority,
            milestone_id: body.milestone_id,
            label_ids: body.label_ids,
        };
        let issue = state
            .service
            .create_tracker_issue(&tenant_id, &organization_id, command, &user_id)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(issue))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn update_tracker_issue<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
    Path(issue_id): Path<String>,
    Json(body): Json<UpdateTrackerIssueRequest>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (tenant_id, organization_id, _) =
            resolve_scope_with_user(&app_ctx, query.tenant_id.as_deref(), query.organization_id.as_deref())?;
        let command = UpdateTrackerIssueCommand {
            title: body.title,
            status: body.status,
            priority: body.priority,
            assignee_id: body.assignee_id,
            milestone_id: body.milestone_id,
            label_ids: body.label_ids,
        };
        let issue = state
            .service
            .update_tracker_issue(&tenant_id, &organization_id, &issue_id, command)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(issue))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn list_tracker_comments<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Path(issue_id): Path<String>,
    Query(query): Query<PageQuery>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let page_result = state
            .service
            .list_tracker_comments(&issue_id, page, page_size)
            .await
            .map_err(map_service_error)?;
        Ok(list_page_data(page_result.items, page_result.page, page_result.page_size, page_result.total))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn create_tracker_comment<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Path(issue_id): Path<String>,
    Json(body): Json<CreateTrackerCommentRequest>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (_, _, user_id) =
            resolve_scope_with_user(&app_ctx, None, None)?;
        let comment = state
            .service
            .create_tracker_comment(&issue_id, &user_id, &body.content)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(comment))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn toggle_tracker_vote<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Path(issue_id): Path<String>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (_, _, user_id) =
            resolve_scope_with_user(&app_ctx, None, None)?;
        let voted = state
            .service
            .toggle_tracker_vote(&issue_id, &user_id)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(VoteResponse { voted }))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn get_tracker_vote_status<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Path(issue_id): Path<String>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (_, _, user_id) =
            resolve_scope_with_user(&app_ctx, None, None)?;
        let voted = state
            .service
            .has_voted(&issue_id, &user_id)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(VoteStatusResponse { voted }))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn list_tracker_labels<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (tenant_id, organization_id, _) =
            resolve_scope_with_user(&app_ctx, query.tenant_id.as_deref(), query.organization_id.as_deref())?;
        let labels = state
            .service
            .list_tracker_labels(&tenant_id, &organization_id)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(TrackerLabelsResponse { labels: labels.into_iter().map(serde_json::to_value).collect::<Result<_, _>>().map_err(|e| ApiProblem::internal_server_error(e.to_string()))? }))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn create_tracker_label<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
    Json(body): Json<CreateTrackerLabelRequest>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (tenant_id, organization_id, _) =
            resolve_scope_with_user(&app_ctx, query.tenant_id.as_deref(), query.organization_id.as_deref())?;
        let label = state
            .service
            .create_tracker_label(&tenant_id, &organization_id, &body.name, body.color.as_deref().unwrap_or("6e7681"), body.description.as_deref())
            .await
            .map_err(map_service_error)?;
        Ok(item_data(label))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn list_tracker_milestones<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (tenant_id, organization_id, _) =
            resolve_scope_with_user(&app_ctx, query.tenant_id.as_deref(), query.organization_id.as_deref())?;
        let milestones = state
            .service
            .list_tracker_milestones(&tenant_id, &organization_id, None)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(TrackerMilestonesResponse { milestones: milestones.into_iter().map(serde_json::to_value).collect::<Result<_, _>>().map_err(|e| ApiProblem::internal_server_error(e.to_string()))? }))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn create_tracker_milestone<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
    Json(body): Json<CreateTrackerMilestoneRequest>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (tenant_id, organization_id, _) =
            resolve_scope_with_user(&app_ctx, query.tenant_id.as_deref(), query.organization_id.as_deref())?;
        let milestone = state
            .service
            .create_tracker_milestone(&tenant_id, &organization_id, &body.title, body.description.as_deref(), body.due_date.as_deref())
            .await
            .map_err(map_service_error)?;
        Ok(item_data(milestone))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn get_tracker_milestone_issues<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Path(milestone_id): Path<String>,
    Query(query): Query<MilestoneIssueListQuery>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (tenant_id, organization_id, _) =
            resolve_scope_with_user(&app_ctx, query.tenant_id.as_deref(), query.organization_id.as_deref())?;
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let page_result = state
            .service
            .get_tracker_milestone_issues(&tenant_id, &organization_id, &milestone_id, page, page_size)
            .await
            .map_err(map_service_error)?;
        Ok(list_page_data(page_result.items, page_result.page, page_result.page_size, page_result.total))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn list_tracker_roadmaps<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (tenant_id, organization_id, _) =
            resolve_scope_with_user(&app_ctx, query.tenant_id.as_deref(), query.organization_id.as_deref())?;
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let page_result = state
            .service
            .list_tracker_roadmaps(&tenant_id, &organization_id, page, page_size)
            .await
            .map_err(map_service_error)?;
        Ok(list_page_data(page_result.items, page_result.page, page_result.page_size, page_result.total))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn create_tracker_roadmap<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
    Json(body): Json<CreateTrackerRoadmapRequest>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (tenant_id, organization_id, _) =
            resolve_scope_with_user(&app_ctx, query.tenant_id.as_deref(), query.organization_id.as_deref())?;
        let roadmap = state
            .service
            .create_tracker_roadmap(&tenant_id, &organization_id, &body.title, body.description.as_deref(), body.start_date.as_deref(), body.target_date.as_deref())
            .await
            .map_err(map_service_error)?;
        Ok(item_data(roadmap))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn get_tracker_roadmap_detail<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
    Path(roadmap_id): Path<String>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (tenant_id, organization_id, _) =
            resolve_scope_with_user(&app_ctx, query.tenant_id.as_deref(), query.organization_id.as_deref())?;
        let view = state
            .service
            .get_tracker_roadmap_detail(&tenant_id, &organization_id, &roadmap_id)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(view))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn update_tracker_roadmap<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Query(query): Query<PageQuery>,
    Path(roadmap_id): Path<String>,
    Json(body): Json<UpdateTrackerRoadmapRequest>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let (tenant_id, organization_id, _) =
            resolve_scope_with_user(&app_ctx, query.tenant_id.as_deref(), query.organization_id.as_deref())?;
        let roadmap = state
            .service
            .update_tracker_roadmap(&tenant_id, &organization_id, &roadmap_id, body.title.as_deref(), body.description.as_ref().map(|d| d.as_deref()), body.status.as_deref(), body.start_date.as_ref().map(|d| d.as_deref()), body.target_date.as_ref().map(|d| d.as_deref()))
            .await
            .map_err(map_service_error)?;
        Ok(item_data(roadmap))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn add_tracker_roadmap_item<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Path(roadmap_id): Path<String>,
    Json(body): Json<CreateTrackerRoadmapItemRequest>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        let command = CreateTrackerRoadmapItemCommand {
            issue_id: body.issue_id,
            track: body.track,
            start_date: body.start_date,
            target_date: body.target_date,
            sort_order: body.sort_order,
        };
        let item = state
            .service
            .add_tracker_roadmap_item(&roadmap_id, command)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(item))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}

pub async fn remove_tracker_roadmap_item<S: GitHubSyncStore + TrackerStore>(
    State(state): State<GitHubAppState<S>>,
    app_ctx: WebRequestContext,
    Path((roadmap_id, item_id)): Path<(String, String)>,
) -> Response {
    let result: ApiResult<_> = (|| async {
        state
            .service
            .remove_tracker_roadmap_item(&roadmap_id, &item_id)
            .await
            .map_err(map_service_error)?;
        Ok(item_data(CommandAcceptedResponse { accepted: true }))
    })()
    .await;
    finish_api_json(&app_ctx, result)
}
