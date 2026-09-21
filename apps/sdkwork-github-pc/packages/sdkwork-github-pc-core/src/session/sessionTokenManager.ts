import { readBootstrapAccessTokenFromProcessEnv } from '@sdkwork/iam-credential-entry';
import type { SessionSnapshot, SessionStore } from './sessionStore';

export interface GithubSessionTokenManager {
  clearTokens(): void;
  getAccessToken(): string | undefined;
  getAuthToken(): string | undefined;
  getRefreshToken(): string | undefined;
  hasToken(): boolean;
  isValid(): boolean;
  setAccessToken(token: string): void;
  setAuthToken(token: string): void;
  setRefreshToken(token: string): void;
}

export function createGithubSessionTokenManager(session: SessionStore): GithubSessionTokenManager {
  return {
    clearTokens() {
      session.clearSession();
    },
    getAccessToken() {
      // Fall back to the private bootstrap Access-Token artifact when no
      // interactive session exists (APP_SDK_INTEGRATION_SPEC section 4).
      return session.getSnapshot().accessToken ?? readBootstrapAccessTokenFromProcessEnv();
    },
    getAuthToken() {
      return session.getSnapshot().authToken;
    },
    getRefreshToken() {
      return session.getSnapshot().refreshToken;
    },
    hasToken() {
      const snapshot = session.getSnapshot();
      return Boolean(snapshot.authToken || snapshot.accessToken);
    },
    isValid() {
      const snapshot = session.getSnapshot();
      return Boolean(snapshot.authToken && snapshot.accessToken);
    },
    setAccessToken(token) {
      mergeSession(session, { accessToken: token });
    },
    setAuthToken(token) {
      mergeSession(session, { authToken: token });
    },
    setRefreshToken(token) {
      mergeSession(session, { refreshToken: token });
    },
  };
}

function mergeSession(session: SessionStore, patch: Partial<SessionSnapshot>): void {
  session.setSession({ ...session.getSnapshot(), ...patch });
}
