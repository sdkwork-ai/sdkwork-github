#!/usr/bin/env node
import { spawn } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { mergeRepoDevBootstrapAccessTokenEnv } from '../../sdkwork-iam/scripts/dev/create-dev-bootstrap-access-token-env.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const profile = resolve(root, 'configs/topology/standalone.unified-process.development.env');
const profileEnv = Object.fromEntries(
  readFileSync(profile, 'utf8')
    .split(/\r?\n/u)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith('#'))
    .map((line) => {
      const index = line.indexOf('=');
      return [line.slice(0, index), line.slice(index + 1)];
    }),
);
const iamRepoRoot = resolve(root, '..', 'sdkwork-iam');
const env = mergeRepoDevBootstrapAccessTokenEnv({
  repoRoot: root,
  manifestPath: 'apps/sdkwork-github-pc/sdkwork.app.config.json',
  appId: 'sdkwork-github-pc',
  env: {
    ...profileEnv,
    SDKWORK_APP_ROOT: root,
    SDKWORK_GITHUB_APP_ROOT: root,
    SDKWORK_IAM_APP_ROOT: iamRepoRoot,
  },
});

const api = spawn('cargo', ['run', '-p', 'sdkwork-api-github-standalone-gateway'], {
  cwd: root,
  env: { ...process.env, ...env },
  stdio: 'inherit',
  shell: true,
});

const web = spawn('pnpm', ['--dir', 'apps/sdkwork-github-pc', 'dev'], {
  cwd: root,
  env: { ...process.env, ...env },
  stdio: 'inherit',
  shell: true,
});

function shutdown() {
  api.kill();
  web.kill();
  process.exit(0);
}

process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);
