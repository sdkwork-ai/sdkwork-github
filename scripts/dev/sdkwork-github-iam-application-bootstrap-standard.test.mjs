import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..', '..');
const iamRepoRoot = path.resolve(repoRoot, '..', 'sdkwork-iam');

function read(relativePath, root = repoRoot) {
  return fs.readFileSync(path.join(root, relativePath), 'utf8');
}

const bootstrapSource = read(
  'crates/sdkwork-api-github-standalone-gateway/src/bootstrap/iam_application_bootstrap.rs',
);
const routersSource = read('crates/sdkwork-api-github-standalone-gateway/src/bootstrap/routers.rs');
const apiServerCargo = read('crates/sdkwork-api-github-standalone-gateway/Cargo.toml');
const workspaceCargo = read('Cargo.toml');
const devScript = read('scripts/github-dev.mjs');
const sharedBootstrapSource = read(
  'crates/sdkwork-iam-embedded-application-bootstrap/src/runtime.rs',
  iamRepoRoot,
);

assert.match(
  bootstrapSource,
  /ensure_tenant_application_from_app_root_with_env_and_fallback/u,
  'Github IAM bootstrap must delegate to the shared embedded bootstrap crate.',
);

assert.match(
  routersSource,
  /ensure_github_tenant_application_bootstrap/u,
  'Github API server must provision tenant applications before building the IAM router.',
);

assert.match(
  routersSource,
  /bootstrap_iam_database_from_env/u,
  'Github API server must bootstrap IAM schema before tenant application provisioning.',
);

assert.match(
  apiServerCargo,
  /sdkwork_iam_embedded_application_bootstrap/u,
  'API server must depend on sdkwork-iam-embedded-application-bootstrap.',
);

assert.match(
  workspaceCargo,
  /sdkwork-iam-embedded-application-bootstrap/u,
  'Workspace must include sdkwork-iam-embedded-application-bootstrap.',
);

assert.match(
  devScript,
  /SDKWORK_APP_ROOT:\s*root/u,
  'Dev script must inject SDKWORK_APP_ROOT for embedded IAM bootstrap.',
);

assert.match(
  devScript,
  /SDKWORK_IAM_APP_ROOT:\s*iamRepoRoot/u,
  'Dev script must inject SDKWORK_IAM_APP_ROOT at the sdkwork-iam repository root for IMF catalog materialization.',
);

assert.match(
  devScript,
  /appId:\s*'sdkwork-github-pc'/u,
  'Dev script must issue bootstrap Access-Token with the PC runtime appId.',
);

assert.match(
  sharedBootstrapSource,
  /SDKWORK_GITHUB_APP_ROOT/u,
  'Shared embedded bootstrap must resolve SDKWORK_GITHUB_APP_ROOT.',
);

console.log('sdkwork-github IAM application bootstrap standard passed.');
