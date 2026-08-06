-- Demo bootstrap data for local development and contract verification.

INSERT INTO github_repository (
    id, tenant_id, organization_id, full_name, owner, description, default_branch, html_url, is_private, created_at, updated_at
) VALUES (
    'github-repo-demo-1',
    '100001',
    '0',
    'sdkwork/github-demo',
    'sdkwork',
    'Demo repository for SDKWork GitHub integration',
    'main',
    'https://github.com/sdkwork/github-demo',
    0,
    '2026-01-01T00:00:00Z',
    '2026-01-01T00:00:00Z'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO github_issue (
    id, tenant_id, organization_id, repository_id, number, title, state, html_url, created_at, updated_at
) VALUES (
    'github-issue-demo-1',
    '100001',
    '0',
    'github-repo-demo-1',
    1,
    'Bootstrap issue',
    'open',
    'https://github.com/sdkwork/github-demo/issues/1',
    '2026-01-01T00:00:00Z',
    '2026-01-01T00:00:00Z'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO github_plan (
    id, tenant_id, organization_id, repository_id, title, status, created_at, updated_at
) VALUES (
    'github-plan-demo-1',
    '100001',
    '0',
    'github-repo-demo-1',
    'Launch checklist',
    'active',
    '2026-01-01T00:00:00Z',
    '2026-01-01T00:00:00Z'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO github_plan_item (
    id, plan_id, title, status, sort_order, issue_id, created_at, updated_at
) VALUES (
    'github-plan-item-demo-1',
    'github-plan-demo-1',
    'Verify bootstrap issue linkage',
    'pending',
    1,
    'github-issue-demo-1',
    '2026-01-01T00:00:00Z',
    '2026-01-01T00:00:00Z'
)
ON CONFLICT (id) DO NOTHING;