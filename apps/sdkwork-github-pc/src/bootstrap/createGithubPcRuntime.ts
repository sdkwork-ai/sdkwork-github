import {
  createGithubAppSdkClient,
  createGithubIamRuntime,
  createGithubSessionTokenManager,
  createRuntimeConfig,
  createSessionStore,
  type GithubPcRuntime,
} from '@sdkwork/github-pc-core';

export function createGithubPcRuntime(): GithubPcRuntime {
  const config = createRuntimeConfig(import.meta.env);
  const session = createSessionStore(resolvePersistentSessionStorage());
  const tokenManager = createGithubSessionTokenManager(session);
  const githubSdk = createGithubAppSdkClient({ config, tokenManager });
  const iamRuntime = createGithubIamRuntime({
    config,
    githubSdk,
    session,
    tokenManager,
  });

  return {
    config,
    githubSdk,
    iamRuntime,
    session,
  };
}

function resolvePersistentSessionStorage(): Storage | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }
  const storageKey = 'sdkwork-github-pc-session';
  const legacySession = window.sessionStorage.getItem(storageKey);
  if (legacySession && !window.localStorage.getItem(storageKey)) {
    window.localStorage.setItem(storageKey, legacySession);
  }
  if (legacySession) {
    window.sessionStorage.removeItem(storageKey);
  }
  return window.localStorage;
}
