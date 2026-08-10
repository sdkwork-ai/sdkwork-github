-- sdkwork:migration
-- id: 0001_organization_id_not_null
-- engine: postgres
-- module: sdkwork-github
-- purpose: Enforce organization_id NOT NULL DEFAULT on all tables in the
--   consolidated baseline. NULL rows (pre-standard data anomalies) are
--   backfilled with the platform sentinel before NOT NULL is set, and
--   NOT NULL columns without an explicit default receive the sentinel
--   default, keeping existing deployments consistent with fresh baseline
--   installs.
-- reversible: false
-- rollback: forward-fix (sentinel backfill is the canonical fix; NULL
--   organization rows are data anomalies)
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE github_repository ADD COLUMN IF NOT EXISTS organization_id TEXT NOT NULL DEFAULT '0';
UPDATE github_repository SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE github_repository ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE github_repository ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE github_issue ADD COLUMN IF NOT EXISTS organization_id TEXT NOT NULL DEFAULT '0';
UPDATE github_issue SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE github_issue ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE github_issue ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE github_plan ADD COLUMN IF NOT EXISTS organization_id TEXT NOT NULL DEFAULT '0';
UPDATE github_plan SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE github_plan ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE github_plan ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE github_provider_account ADD COLUMN IF NOT EXISTS organization_id TEXT NOT NULL DEFAULT '0';
UPDATE github_provider_account SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE github_provider_account ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE github_provider_account ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE github_oauth_pending ADD COLUMN IF NOT EXISTS organization_id TEXT NOT NULL DEFAULT '0';
UPDATE github_oauth_pending SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE github_oauth_pending ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE github_oauth_pending ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE github_tracker_label ADD COLUMN IF NOT EXISTS organization_id TEXT NOT NULL DEFAULT '0';
UPDATE github_tracker_label SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE github_tracker_label ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE github_tracker_label ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE github_tracker_milestone ADD COLUMN IF NOT EXISTS organization_id TEXT NOT NULL DEFAULT '0';
UPDATE github_tracker_milestone SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE github_tracker_milestone ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE github_tracker_milestone ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE github_tracker_issue ADD COLUMN IF NOT EXISTS organization_id TEXT NOT NULL DEFAULT '0';
UPDATE github_tracker_issue SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE github_tracker_issue ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE github_tracker_issue ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE github_tracker_roadmap ADD COLUMN IF NOT EXISTS organization_id TEXT NOT NULL DEFAULT '0';
UPDATE github_tracker_roadmap SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE github_tracker_roadmap ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE github_tracker_roadmap ALTER COLUMN organization_id SET NOT NULL;

COMMIT;
