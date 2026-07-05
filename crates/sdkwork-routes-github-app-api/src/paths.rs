pub const REPOSITORIES: &str = "/app/v3/api/github/repositories";
pub const REPOSITORIES_SYNC: &str = "/app/v3/api/github/repositories/sync";
pub const ISSUES: &str = "/app/v3/api/github/issues";
pub const ISSUES_SYNC: &str = "/app/v3/api/github/issues/sync";
pub const PLANS: &str = "/app/v3/api/github/plans";
pub const CATALOG_BOOTSTRAP: &str = "/app/v3/api/github/catalog/bootstrap";
pub const INTEGRATION: &str = "/app/v3/api/github/integration";
pub const INTEGRATION_OAUTH_BEGIN: &str = "/app/v3/api/github/integration/oauth/begin";
pub const INTEGRATION_OAUTH_CALLBACK: &str = "/app/v3/api/github/integration/oauth/callback";

// Tracker paths
pub const TRACKER_ISSUES: &str = "/app/v3/api/github/tracker/issues";
pub const TRACKER_ISSUE_DETAIL: &str = "/app/v3/api/github/tracker/issues/:issue_id";
pub const TRACKER_ISSUE_COMMENTS: &str = "/app/v3/api/github/tracker/issues/:issue_id/comments";
pub const TRACKER_ISSUE_VOTES: &str = "/app/v3/api/github/tracker/issues/:issue_id/votes";
pub const TRACKER_ISSUE_VOTE_STATUS: &str = "/app/v3/api/github/tracker/issues/:issue_id/votes/status";
pub const TRACKER_LABELS: &str = "/app/v3/api/github/tracker/labels";
pub const TRACKER_MILESTONES: &str = "/app/v3/api/github/tracker/milestones";
pub const TRACKER_MILESTONE_ISSUES: &str = "/app/v3/api/github/tracker/milestones/:milestone_id/issues";
pub const TRACKER_ROADMAPS: &str = "/app/v3/api/github/tracker/roadmaps";
pub const TRACKER_ROADMAP_DETAIL: &str = "/app/v3/api/github/tracker/roadmaps/:roadmap_id";
pub const TRACKER_ROADMAP_ITEMS: &str = "/app/v3/api/github/tracker/roadmaps/:roadmap_id/items";
pub const TRACKER_ROADMAP_ITEM_DETAIL: &str = "/app/v3/api/github/tracker/roadmaps/:roadmap_id/items/:item_id";
