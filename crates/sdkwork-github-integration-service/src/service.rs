use sdkwork_utils_rust::string::is_blank;

use crate::domain::{
    CatalogBootstrapResult, IntegrationStatus, Issue, LinkIntegrationCommand, Page, Plan,
    PlanItem, PlanView, Repository, SyncResult,
};
use crate::domain::{
    CreateTrackerIssueCommand, CreateTrackerRoadmapItemCommand, MilestoneProgress,
    TrackerComment, TrackerIssue, TrackerIssueQuery, TrackerIssueView, TrackerLabel,
    TrackerMilestone, TrackerRoadmap, TrackerRoadmapItem, TrackerRoadmapItemView,
    TrackerRoadmapView, UpdateTrackerIssueCommand,
};
use crate::error::ServiceError;
use crate::ports::{GitHubStore, GitHubSyncStore, TrackerStore};

const GITHUB_PROVIDER: &str = "github";

pub struct GitHubIntegrationService<S: GitHubStore> {
    store: S,
}

impl<S: GitHubStore + Clone> Clone for GitHubIntegrationService<S> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
        }
    }
}

impl<S: GitHubStore> GitHubIntegrationService<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn list_repositories(
        &self,
        tenant_id: &str,
        organization_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Page<Repository>, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        self.store
            .list_repositories(tenant_id, organization_id, page, page_size)
            .await
    }

    pub async fn list_issues(
        &self,
        tenant_id: &str,
        organization_id: &str,
        repository_id: Option<&str>,
        page: u32,
        page_size: u32,
    ) -> Result<Page<Issue>, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        self.store
            .list_issues(tenant_id, organization_id, repository_id, page, page_size)
            .await
    }

    pub async fn list_plans(
        &self,
        tenant_id: &str,
        organization_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Page<PlanView>, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        let page_result = self
            .store
            .list_plans(tenant_id, organization_id, page, page_size)
            .await?;
        let plan_ids: Vec<String> = page_result.items.iter().map(|plan| plan.id.clone()).collect();
        let mut items = self.store.list_plan_items_for_plan_ids(&plan_ids).await?;
        items.sort_by(|left, right| {
            left.plan_id
                .cmp(&right.plan_id)
                .then(left.sort_order.cmp(&right.sort_order))
        });
        let mut items_by_plan: std::collections::HashMap<String, Vec<PlanItem>> =
            std::collections::HashMap::new();
        for item in items {
            items_by_plan
                .entry(item.plan_id.clone())
                .or_default()
                .push(item);
        }
        Ok(Page {
            items: page_result
                .items
                .into_iter()
                .map(|plan| {
                    let plan_items = items_by_plan.remove(&plan.id).unwrap_or_default();
                    PlanView::from_plan(plan, plan_items)
                })
                .collect(),
            page: page_result.page,
            page_size: page_result.page_size,
            total: page_result.total,
        })
    }
}

impl<S: GitHubSyncStore> GitHubIntegrationService<S> {
    pub async fn get_integration_status(
        &self,
        tenant_id: &str,
        organization_id: &str,
    ) -> Result<IntegrationStatus, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        self.store
            .get_integration_status(tenant_id, organization_id, GITHUB_PROVIDER)
            .await
    }

    pub async fn link_integration(
        &self,
        tenant_id: &str,
        organization_id: &str,
        mut command: LinkIntegrationCommand,
    ) -> Result<IntegrationStatus, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        if is_blank(Some(command.access_token.as_str())) {
            return Err(ServiceError::Validation(
                "access_token is required".to_string(),
            ));
        }

        command = enrich_link_command(command).await?;

        let cipher = sdkwork_github_integration_provider_github::GitHubCredentialCipher::from_env()
            .map_err(|error| ServiceError::Configuration(error.to_string()))?;
        let encrypted = cipher
            .encrypt(&command.access_token)
            .map_err(|error| ServiceError::Configuration(error.to_string()))?;

        self.store
            .link_integration(
                tenant_id,
                organization_id,
                GITHUB_PROVIDER,
                &command,
                &encrypted,
            )
            .await
    }

    pub async fn unlink_integration(
        &self,
        tenant_id: &str,
        organization_id: &str,
    ) -> Result<IntegrationStatus, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        self.store
            .revoke_provider_account(tenant_id, organization_id, GITHUB_PROVIDER)
            .await?;
        self.get_integration_status(tenant_id, organization_id)
            .await
    }

    pub async fn begin_oauth_integration(
        &self,
        tenant_id: &str,
        organization_id: &str,
    ) -> Result<crate::domain::OAuthBeginResult, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        let oauth = sdkwork_github_integration_provider_github::GitHubOAuthClient::from_env()
            .map_err(|error| ServiceError::Configuration(error.to_string()))?;
        self.store.purge_expired_oauth_pending().await?;
        let state = uuid::Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(10);
        self.store
            .create_oauth_pending(&state, tenant_id, organization_id, expires_at)
            .await?;
        Ok(crate::domain::OAuthBeginResult {
            provider: GITHUB_PROVIDER.to_string(),
            authorization_url: oauth.build_authorization_url(&state),
            state,
        })
    }

    pub async fn complete_oauth_integration(
        &self,
        state: &str,
        code: &str,
    ) -> Result<IntegrationStatus, ServiceError> {
        if is_blank(Some(state)) || is_blank(Some(code)) {
            return Err(ServiceError::Validation(
                "state and code are required".to_string(),
            ));
        }
        let (tenant_id, organization_id) = self
            .store
            .consume_oauth_pending(state)
            .await?
            .ok_or_else(|| ServiceError::Validation("oauth state is invalid or expired".to_string()))?;
        let oauth = sdkwork_github_integration_provider_github::GitHubOAuthClient::from_env()
            .map_err(|error| ServiceError::Configuration(error.to_string()))?;
        let exchange = oauth
            .exchange_code(code)
            .await
            .map_err(|error| ServiceError::Integration(error.to_string()))?;
        let scopes = exchange
            .scope
            .filter(|value| !value.trim().is_empty())
            .or_else(|| Some(oauth.configured_scopes().to_string()));
        self.link_integration(
            &tenant_id,
            &organization_id,
            LinkIntegrationCommand {
                access_token: exchange.access_token,
                external_account_id: None,
                scopes,
            },
        )
        .await
    }

    pub async fn list_admin_integrations(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Page<crate::domain::AdminIntegrationView>, ServiceError> {
        self.store.list_admin_integrations(page, page_size).await
    }

    pub async fn bootstrap_notable_catalog(
        &self,
        tenant_id: &str,
        organization_id: &str,
    ) -> Result<CatalogBootstrapResult, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        let catalog = crate::catalog::load_notable_repository_catalog(
            &crate::catalog::resolve_catalog_app_root(),
        )?;
        let public_api = sdkwork_github_integration_provider_github::GitHubPublicApiClient::new();
        let provider = self.resolve_provider(tenant_id, organization_id).await.ok();
        let now = chrono::Utc::now();
        let mut result = CatalogBootstrapResult {
            repositories_synced: 0,
            issues_synced: 0,
            plans_created: 0,
            plan_items_created: 0,
        };

        for (index, entry) in catalog.repositories.into_iter().enumerate() {
            if index > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }

            let remote = match public_api
                .fetch_repository(&entry.owner, &entry.name)
                .await
            {
                Ok(remote) => remote,
                Err(error) => {
                    tracing::warn!(
                        owner = entry.owner.as_str(),
                        repo = entry.name.as_str(),
                        error = %error,
                        "catalog repository fetch skipped"
                    );
                    continue;
                }
            };
            let repository = Repository {
                id: format!("github-repo-{}", remote.id),
                tenant_id: tenant_id.to_string(),
                organization_id: organization_id.to_string(),
                full_name: remote.full_name.clone(),
                owner: remote.owner.login.clone(),
                description: remote.description,
                default_branch: remote.default_branch,
                html_url: remote.html_url,
                is_private: remote.private,
                created_at: now,
                updated_at: now,
            };
            self.store.upsert_repository(&repository).await?;
            result.repositories_synced += 1;

            let mut linked_issue_id = None;
            if entry.sync_issues && entry.max_issues > 0 {
                if let Some(provider) = provider.as_ref() {
                    match provider
                        .fetch_issues(&entry.owner, &entry.name)
                        .await
                    {
                        Ok(remote_issues) => {
                            for (index, remote_issue) in remote_issues
                                .into_iter()
                                .take(entry.max_issues as usize)
                                .enumerate()
                            {
                                let issue = Issue {
                                    id: format!("github-issue-{}", remote_issue.id),
                                    tenant_id: tenant_id.to_string(),
                                    organization_id: organization_id.to_string(),
                                    repository_id: repository.id.clone(),
                                    number: remote_issue.number,
                                    title: remote_issue.title,
                                    state: remote_issue.state,
                                    html_url: remote_issue.html_url,
                                    created_at: now,
                                    updated_at: now,
                                };
                                if index == 0 {
                                    linked_issue_id = Some(issue.id.clone());
                                }
                                self.store.upsert_issue(&issue).await?;
                                result.issues_synced += 1;
                            }
                        }
                        Err(error) => {
                            tracing::warn!(
                                owner = entry.owner.as_str(),
                                repo = entry.name.as_str(),
                                error = %error,
                                "catalog issue sync skipped for repository"
                            );
                        }
                    }
                } else {
                    tracing::info!(
                        owner = entry.owner.as_str(),
                        repo = entry.name.as_str(),
                        "catalog issue sync skipped because GitHub provider is not configured"
                    );
                }
            }

            let plan_id = format!("github-plan-catalog-{}-{}", entry.owner, entry.name);
            let plan = Plan {
                id: plan_id.clone(),
                tenant_id: tenant_id.to_string(),
                organization_id: organization_id.to_string(),
                repository_id: Some(repository.id.clone()),
                title: entry.plan_title.clone(),
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
            };
            self.store.upsert_plan(&plan).await?;
            result.plans_created += 1;

            let checklist = [
                "Review README and contribution guide",
                "Monitor high-priority open issues",
                "Track release milestones",
            ];
            for (sort_order, title) in checklist.iter().enumerate() {
                let item = PlanItem {
                    id: format!("{plan_id}-item-{sort_order}"),
                    plan_id: plan_id.clone(),
                    title: (*title).to_string(),
                    status: "pending".to_string(),
                    sort_order: sort_order as i32,
                    issue_id: if sort_order == 1 {
                        linked_issue_id.clone()
                    } else {
                        None
                    },
                    created_at: now,
                    updated_at: now,
                };
                self.store.upsert_plan_item(&item).await?;
                result.plan_items_created += 1;
            }
        }

        Ok(result)
    }

    pub async fn sync_repositories(
        &self,
        tenant_id: &str,
        organization_id: &str,
    ) -> Result<SyncResult, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        let provider = self
            .resolve_provider(tenant_id, organization_id)
            .await?;

        let remote_repositories = provider
            .fetch_repositories()
            .await
            .map_err(|error| ServiceError::Integration(error.to_string()))?;

        let now = chrono::Utc::now();
        let mut synced_count = 0_u64;
        for remote in remote_repositories {
            let repository = Repository {
                id: format!("github-repo-{}", remote.id),
                tenant_id: tenant_id.to_string(),
                organization_id: organization_id.to_string(),
                full_name: remote.full_name,
                owner: remote.owner.login,
                description: remote.description,
                default_branch: remote.default_branch,
                html_url: remote.html_url,
                is_private: remote.private,
                created_at: now,
                updated_at: now,
            };
            self.store.upsert_repository(&repository).await?;
            synced_count += 1;
        }

        self.store
            .touch_provider_last_synced(tenant_id, organization_id, GITHUB_PROVIDER)
            .await?;

        Ok(SyncResult {
            provider: provider.provider_key().to_string(),
            synced_count,
        })
    }

    pub async fn sync_issues(
        &self,
        tenant_id: &str,
        organization_id: &str,
        repository_id: Option<&str>,
    ) -> Result<SyncResult, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        let provider = self
            .resolve_provider(tenant_id, organization_id)
            .await?;

        let repositories = if let Some(repository_id) = repository_id {
            let page = self
                .store
                .list_repositories(tenant_id, organization_id, 1, 100)
                .await?;
            page.items
                .into_iter()
                .filter(|item| item.id == repository_id)
                .collect::<Vec<_>>()
        } else {
            self.store
                .list_repositories(tenant_id, organization_id, 1, 100)
                .await?
                .items
        };

        if repositories.is_empty() {
            return Err(ServiceError::Validation(
                "sync issues requires at least one tracked repository".to_string(),
            ));
        }

        let now = chrono::Utc::now();
        let mut synced_count = 0_u64;
        for repository in repositories {
            let (owner, repo) = split_full_name(&repository.full_name)?;
            let remote_issues = provider
                .fetch_issues(owner, repo)
                .await
                .map_err(|error| ServiceError::Integration(error.to_string()))?;

            for remote in remote_issues {
                let issue = Issue {
                    id: format!("github-issue-{}", remote.id),
                    tenant_id: tenant_id.to_string(),
                    organization_id: organization_id.to_string(),
                    repository_id: repository.id.clone(),
                    number: remote.number,
                    title: remote.title,
                    state: remote.state,
                    html_url: remote.html_url,
                    created_at: now,
                    updated_at: now,
                };
                self.store.upsert_issue(&issue).await?;
                synced_count += 1;
            }
        }

        self.store
            .touch_provider_last_synced(tenant_id, organization_id, GITHUB_PROVIDER)
            .await?;

        Ok(SyncResult {
            provider: provider.provider_key().to_string(),
            synced_count,
        })
    }

    async fn resolve_provider(
        &self,
        tenant_id: &str,
        organization_id: &str,
    ) -> Result<sdkwork_github_integration_provider_github::GitHubRestProvider, ServiceError> {
        if let Some(account) = self
            .store
            .find_active_provider_account(tenant_id, organization_id, GITHUB_PROVIDER)
            .await?
        {
            let cipher =
                sdkwork_github_integration_provider_github::GitHubCredentialCipher::from_env()
                    .map_err(|error| ServiceError::Configuration(error.to_string()))?;
            let token = cipher
                .decrypt(&account.access_token_cipher)
                .map_err(|error| ServiceError::Configuration(error.to_string()))?;
            return Ok(build_provider(token));
        }

        if let Some(provider) =
            sdkwork_github_integration_provider_github::GitHubRestProvider::from_env()
        {
            tracing::warn!(
                tenant_id,
                organization_id,
                "using SDKWORK_GITHUB_INTEGRATION_PAT fallback; link tenant integration for production"
            );
            return Ok(provider);
        }

        Err(ServiceError::Configuration(
            "GitHub integration is not linked; configure tenant OAuth/PAT linking before sync"
                .to_string(),
        ))
    }
}

// ════════════════════════════════════════════════════════
// Tracker service methods
// ════════════════════════════════════════════════════════

impl<S: GitHubStore + TrackerStore> GitHubIntegrationService<S> {
    pub async fn list_tracker_issues(
        &self,
        tenant_id: &str,
        organization_id: &str,
        query: &TrackerIssueQuery,
        page: u32,
        page_size: u32,
    ) -> Result<Page<TrackerIssueView>, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        let page_result = self
            .store
            .list_tracker_issues(tenant_id, organization_id, query, page, page_size)
            .await?;
        let issue_ids: Vec<String> = page_result.items.iter().map(|i| i.id.clone()).collect();
        let label_pairs = self.store.list_labels_for_issue_ids(&issue_ids).await?;
        let mut labels_by_issue: std::collections::HashMap<String, Vec<TrackerLabel>> =
            std::collections::HashMap::new();
        for (issue_id, label) in label_pairs {
            labels_by_issue.entry(issue_id).or_default().push(label);
        }
        let milestone_ids: Vec<String> = page_result
            .items
            .iter()
            .filter_map(|i| i.milestone_id.clone())
            .collect::<Vec<_>>();
        let mut milestone_map: std::collections::HashMap<String, TrackerMilestone> =
            std::collections::HashMap::new();
        for mid in &milestone_ids {
            if let Some(m) = self.store.get_milestone_for_issue(mid).await? {
                milestone_map.insert(mid.clone(), m);
            }
        }
        Ok(Page {
            items: page_result
                .items
                .into_iter()
                .map(|issue| {
                    let labels = labels_by_issue.remove(&issue.id).unwrap_or_default();
                    let milestone = issue
                        .milestone_id
                        .as_ref()
                        .and_then(|mid| milestone_map.get(mid).cloned());
                    TrackerIssueView {
                        issue,
                        labels,
                        milestone,
                    }
                })
                .collect(),
            page: page_result.page,
            page_size: page_result.page_size,
            total: page_result.total,
        })
    }

    pub async fn get_tracker_issue_detail(
        &self,
        tenant_id: &str,
        organization_id: &str,
        issue_id: &str,
    ) -> Result<TrackerIssueView, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        let issue = self
            .store
            .get_tracker_issue(tenant_id, organization_id, issue_id)
            .await?;
        let label_pairs = self.store.list_labels_for_issue_ids(&[issue_id.to_string()]).await?;
        let labels: Vec<TrackerLabel> = label_pairs.into_iter().map(|(_, l)| l).collect();
        let milestone = if let Some(ref mid) = issue.milestone_id {
            self.store.get_milestone_for_issue(mid).await?
        } else {
            None
        };
        Ok(TrackerIssueView {
            issue,
            labels,
            milestone,
        })
    }

    pub async fn create_tracker_issue(
        &self,
        tenant_id: &str,
        organization_id: &str,
        command: CreateTrackerIssueCommand,
        submitted_by: &str,
    ) -> Result<TrackerIssue, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        if command.title.trim().is_empty() {
            return Err(ServiceError::Validation("title is required".to_string()));
        }
        if command.description.trim().is_empty() {
            return Err(ServiceError::Validation("description is required".to_string()));
        }
        let valid_types = ["bug", "feature", "enhancement", "question", "task"];
        if !valid_types.contains(&command.issue_type.as_str()) {
            return Err(ServiceError::Validation(format!(
                "invalid issue type: {}; expected one of {:?}",
                command.issue_type, valid_types
            )));
        }
        self.store
            .create_tracker_issue(tenant_id, organization_id, &command, submitted_by)
            .await
    }

    pub async fn update_tracker_issue(
        &self,
        tenant_id: &str,
        organization_id: &str,
        issue_id: &str,
        command: UpdateTrackerIssueCommand,
    ) -> Result<TrackerIssue, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        self.store
            .update_tracker_issue(tenant_id, organization_id, issue_id, &command)
            .await
    }

    pub async fn list_tracker_comments(
        &self,
        issue_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Page<TrackerComment>, ServiceError> {
        self.store.list_tracker_comments(issue_id, page, page_size).await
    }

    pub async fn create_tracker_comment(
        &self,
        issue_id: &str,
        author_id: &str,
        content: &str,
    ) -> Result<TrackerComment, ServiceError> {
        if content.trim().is_empty() {
            return Err(ServiceError::Validation("content is required".to_string()));
        }
        self.store.create_tracker_comment(issue_id, author_id, content).await
    }

    pub async fn toggle_tracker_vote(
        &self,
        issue_id: &str,
        user_id: &str,
    ) -> Result<bool, ServiceError> {
        self.store.toggle_tracker_vote(issue_id, user_id).await
    }

    pub async fn has_voted(&self, issue_id: &str, user_id: &str) -> Result<bool, ServiceError> {
        self.store.has_voted(issue_id, user_id).await
    }

    pub async fn list_tracker_labels(
        &self,
        tenant_id: &str,
        organization_id: &str,
    ) -> Result<Vec<TrackerLabel>, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        self.store.list_tracker_labels(tenant_id, organization_id).await
    }

    pub async fn create_tracker_label(
        &self,
        tenant_id: &str,
        organization_id: &str,
        name: &str,
        color: &str,
        description: Option<&str>,
    ) -> Result<TrackerLabel, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        if name.trim().is_empty() {
            return Err(ServiceError::Validation("label name is required".to_string()));
        }
        self.store
            .create_tracker_label(tenant_id, organization_id, name, color, description)
            .await
    }

    pub async fn list_tracker_milestones(
        &self,
        tenant_id: &str,
        organization_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<MilestoneProgress>, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        self.store
            .list_tracker_milestones(tenant_id, organization_id, status)
            .await
    }

    pub async fn create_tracker_milestone(
        &self,
        tenant_id: &str,
        organization_id: &str,
        title: &str,
        description: Option<&str>,
        due_date: Option<&str>,
    ) -> Result<TrackerMilestone, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        if title.trim().is_empty() {
            return Err(ServiceError::Validation("milestone title is required".to_string()));
        }
        self.store
            .create_tracker_milestone(tenant_id, organization_id, title, description, due_date)
            .await
    }

    pub async fn get_tracker_milestone_issues(
        &self,
        tenant_id: &str,
        organization_id: &str,
        milestone_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Page<TrackerIssue>, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        self.store
            .get_tracker_milestone_issues(tenant_id, organization_id, milestone_id, page, page_size)
            .await
    }

    pub async fn list_tracker_roadmaps(
        &self,
        tenant_id: &str,
        organization_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Page<TrackerRoadmap>, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        self.store
            .list_tracker_roadmaps(tenant_id, organization_id, page, page_size)
            .await
    }

    pub async fn create_tracker_roadmap(
        &self,
        tenant_id: &str,
        organization_id: &str,
        title: &str,
        description: Option<&str>,
        start_date: Option<&str>,
        target_date: Option<&str>,
    ) -> Result<TrackerRoadmap, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        if title.trim().is_empty() {
            return Err(ServiceError::Validation("roadmap title is required".to_string()));
        }
        self.store
            .create_tracker_roadmap(tenant_id, organization_id, title, description, start_date, target_date)
            .await
    }

    pub async fn get_tracker_roadmap_detail(
        &self,
        tenant_id: &str,
        organization_id: &str,
        roadmap_id: &str,
    ) -> Result<TrackerRoadmapView, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        let roadmap = self
            .store
            .get_tracker_roadmap(tenant_id, organization_id, roadmap_id)
            .await?;
        let items = self.store.list_tracker_roadmap_items(roadmap_id).await?;
        let issue_ids: Vec<String> = items.iter().map(|i| i.issue_id.clone()).collect();
        let label_pairs = self.store.list_labels_for_issue_ids(&issue_ids).await?;
        let mut labels_by_issue: std::collections::HashMap<String, Vec<TrackerLabel>> =
            std::collections::HashMap::new();
        for (issue_id, label) in label_pairs {
            labels_by_issue.entry(issue_id).or_default().push(label);
        }
        let mut item_views = Vec::with_capacity(items.len());
        for item in items {
            let issue = self
                .store
                .get_tracker_issue(tenant_id, organization_id, &item.issue_id)
                .await?;
            let labels = labels_by_issue.remove(&item.issue_id).unwrap_or_default();
            item_views.push(TrackerRoadmapItemView {
                item,
                issue,
                labels,
            });
        }
        Ok(TrackerRoadmapView {
            roadmap,
            items: item_views,
        })
    }

    pub async fn update_tracker_roadmap(
        &self,
        tenant_id: &str,
        organization_id: &str,
        roadmap_id: &str,
        title: Option<&str>,
        description: Option<Option<&str>>,
        status: Option<&str>,
        start_date: Option<Option<&str>>,
        target_date: Option<Option<&str>>,
    ) -> Result<TrackerRoadmap, ServiceError> {
        validate_scope(tenant_id, organization_id)?;
        self.store
            .update_tracker_roadmap(tenant_id, organization_id, roadmap_id, title, description, status, start_date, target_date)
            .await
    }

    pub async fn add_tracker_roadmap_item(
        &self,
        roadmap_id: &str,
        command: CreateTrackerRoadmapItemCommand,
    ) -> Result<TrackerRoadmapItem, ServiceError> {
        self.store.add_tracker_roadmap_item(roadmap_id, &command).await
    }

    pub async fn remove_tracker_roadmap_item(
        &self,
        roadmap_id: &str,
        item_id: &str,
    ) -> Result<(), ServiceError> {
        self.store.remove_tracker_roadmap_item(roadmap_id, item_id).await
    }
}

async fn enrich_link_command(
    mut command: LinkIntegrationCommand,
) -> Result<LinkIntegrationCommand, ServiceError> {
    if command
        .external_account_id
        .as_ref()
        .is_none_or(|value| is_blank(Some(value.as_str())))
    {
        let provider = build_provider(command.access_token.clone());
        let user = provider
            .fetch_current_user()
            .await
            .map_err(|error| ServiceError::Integration(error.to_string()))?;
        command.external_account_id = Some(user.id.to_string());
    }
    Ok(command)
}

fn build_provider(token: String) -> sdkwork_github_integration_provider_github::GitHubRestProvider {
    let api_base = std::env::var("SDKWORK_GITHUB_INTEGRATION_API_BASE")
        .unwrap_or_else(|_| "https://api.github.com".to_string());
    sdkwork_github_integration_provider_github::GitHubRestProvider::new(token, api_base)
}

fn validate_scope(tenant_id: &str, organization_id: &str) -> Result<(), ServiceError> {
    if is_blank(Some(tenant_id)) || is_blank(Some(organization_id)) {
        return Err(ServiceError::Validation(
            "tenant_id and organization_id are required".to_string(),
        ));
    }
    Ok(())
}

fn split_full_name(full_name: &str) -> Result<(&str, &str), ServiceError> {
    let (owner, repo) = full_name.split_once('/').ok_or_else(|| {
        ServiceError::Validation(format!("invalid repository full_name: {full_name}"))
    })?;
    if is_blank(Some(owner)) || is_blank(Some(repo)) {
        return Err(ServiceError::Validation(format!(
            "invalid repository full_name: {full_name}"
        )));
    }
    Ok((owner, repo))
}

#[cfg(test)]
mod tests {
    use super::{split_full_name, validate_scope};
    use crate::error::ServiceError;

    #[test]
    fn rejects_blank_scope() {
        let error = validate_scope("", "org").unwrap_err();
        assert!(matches!(error, ServiceError::Validation(_)));
    }

    #[test]
    fn splits_repository_full_name() {
        let (owner, repo) = split_full_name("sdkwork/demo").unwrap();
        assert_eq!(owner, "sdkwork");
        assert_eq!(repo, "demo");
    }
}
