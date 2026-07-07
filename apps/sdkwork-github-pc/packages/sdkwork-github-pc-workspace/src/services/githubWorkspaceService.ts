import type { GithubAppSdkClient, SessionSnapshot } from '@sdkwork/github-pc-core';

type GithubWorkspaceSdkClient = GithubAppSdkClient['client'];

function resolveScope(context: SessionSnapshot['context']) {
  if (!context?.tenantId) {
    throw new Error('tenant context is required');
  }
  return {
    tenantId: context.tenantId,
    organizationId: context.organizationId,
  };
}

export async function listRepositories(
  client: GithubWorkspaceSdkClient,
  context: SessionSnapshot['context'],
  page = 1,
  pageSize = 20,
) {
  const scope = resolveScope(context);
  return client.github.repositories.list({
    ...scope,
    page,
    pageSize,
  });
}

export async function listIssues(
  client: GithubWorkspaceSdkClient,
  context: SessionSnapshot['context'],
  repositoryId?: string,
  page = 1,
  pageSize = 20,
) {
  const scope = resolveScope(context);
  return client.github.issues.list({
    ...scope,
    repositoryId,
    page,
    pageSize,
  });
}

export async function syncRepositories(
  client: GithubWorkspaceSdkClient,
  context: SessionSnapshot['context'],
) {
  const scope = resolveScope(context);
  return client.github.repositories.sync(scope);
}

export async function syncIssues(
  client: GithubWorkspaceSdkClient,
  context: SessionSnapshot['context'],
  repositoryId?: string,
) {
  const scope = resolveScope(context);
  return client.github.issues.sync({
    ...scope,
    repositoryId,
  });
}

export async function getIntegrationStatus(
  client: GithubWorkspaceSdkClient,
  context: SessionSnapshot['context'],
) {
  const scope = resolveScope(context);
  return client.github.integration.status(scope);
}

export async function linkIntegration(
  client: GithubWorkspaceSdkClient,
  context: SessionSnapshot['context'],
  accessToken: string,
  externalAccountId?: string,
) {
  const scope = resolveScope(context);
  return client.github.integration.link(
    {
      access_token: accessToken,
      external_account_id: externalAccountId,
    },
    scope,
  );
}

export async function unlinkIntegration(
  client: GithubWorkspaceSdkClient,
  context: SessionSnapshot['context'],
) {
  const scope = resolveScope(context);
  return client.github.integration.unlink(scope);
}

export async function beginOAuthIntegration(
  client: GithubWorkspaceSdkClient,
  context: SessionSnapshot['context'],
) {
  const scope = resolveScope(context);
  return client.github.integration.oauth.begin(scope);
}

export async function listPlans(
  client: GithubWorkspaceSdkClient,
  context: SessionSnapshot['context'],
  page = 1,
  pageSize = 20,
) {
  const scope = resolveScope(context);
  return client.github.plans.list({
    ...scope,
    page,
    pageSize,
  });
}

export async function bootstrapNotableCatalog(
  client: GithubWorkspaceSdkClient,
  context: SessionSnapshot['context'],
) {
  const scope = resolveScope(context);
  return client.github.catalog.bootstrap(scope);
}
