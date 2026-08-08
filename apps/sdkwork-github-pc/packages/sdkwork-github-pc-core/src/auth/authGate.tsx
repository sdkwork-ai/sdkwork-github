import React, { useEffect, useMemo, useState } from 'react';
import {
  hasGithubIamSession,
  type SessionSnapshot,
  type SessionStore,
} from '../session/sessionStore';

export interface GithubAuthLocationLike {
  pathname: string;
  search?: string;
  hash?: string;
}

export type GithubAuthGateDecision =
  | { kind: 'product-route' }
  | { kind: 'auth-route' }
  | { kind: 'redirect'; replace: true; to: string };

export interface GithubAuthGateProps {
  authRoutes?: React.ReactNode;
  children: React.ReactNode;
  location?: GithubAuthLocationLike;
  navigate?: (to: string, options: { replace: true }) => void;
  session: SessionStore;
}

const AUTH_LOGIN_PATH = '/auth/login';

export function GithubAuthGate({
  authRoutes,
  children,
  location,
  navigate,
  session,
}: GithubAuthGateProps) {
  const [snapshot, setSnapshot] = useState<SessionSnapshot>(() => session.getSnapshot());
  const currentLocation = useBrowserLocation(location);

  useEffect(() => session.subscribe(setSnapshot), [session]);

  const decision = useMemo(
    () =>
      resolveGithubAuthGateDecision({
        hasSession: hasGithubIamSession(snapshot),
        location: currentLocation,
      }),
    [currentLocation, snapshot],
  );

  useEffect(() => {
    if (decision.kind !== 'redirect') return;
    if (navigate) {
      navigate(decision.to, { replace: true });
      return;
    }
    if (typeof window !== 'undefined') {
      window.location.replace(decision.to);
    }
  }, [decision, navigate]);

  if (decision.kind === 'redirect') return null;
  if (decision.kind === 'auth-route') {
    return React.createElement(React.Fragment, null, authRoutes);
  }
  return React.createElement(React.Fragment, null, children);
}

export function resolveGithubAuthGateDecision({
  hasSession,
  location,
}: {
  hasSession: boolean;
  location: GithubAuthLocationLike;
}): GithubAuthGateDecision {
  const pathname = normalizePathname(location.pathname);
  if (isGithubAuthRoute(pathname)) {
    if (!hasSession) return { kind: 'auth-route' };
    const redirect = new URLSearchParams((location.search ?? '').replace(/^\?/, '')).get('redirect');
    return {
      kind: 'redirect',
      replace: true,
      to: sanitizeRedirect(redirect) || '/',
    };
  }
  if (!hasSession) {
    const returnPath = `${pathname}${location.search ?? ''}${location.hash ?? ''}`;
    return {
      kind: 'redirect',
      replace: true,
      to: `${AUTH_LOGIN_PATH}?redirect=${encodeURIComponent(returnPath)}`,
    };
  }
  return { kind: 'product-route' };
}

function isGithubAuthRoute(pathname: string): boolean {
  return pathname === '/auth' || pathname.startsWith('/auth/');
}

function sanitizeRedirect(value: string | null | undefined): string {
  if (!value) return '/';
  let decoded = value;
  try {
    decoded = decodeURIComponent(value);
  } catch {
    return '/';
  }
  if (!decoded.startsWith('/') || decoded.startsWith('//')) return '/';
  const target = new URL(decoded, 'http://sdkwork-github.local');
  // A redirect back into the auth surface (possibly nested/encoded) must be
  // rejected so the login page does not bounce through itself.
  if (isGithubAuthRoute(target.pathname)) return '/';
  return `${target.pathname}${target.search}${target.hash}`;
}

function normalizePathname(pathname: string): string {
  const normalized = pathname.trim();
  if (!normalized) return '/';
  return normalized.startsWith('/') ? normalized : `/${normalized}`;
}

function useBrowserLocation(location?: GithubAuthLocationLike): GithubAuthLocationLike {
  const [browserLocation, setBrowserLocation] = useState<GithubAuthLocationLike>(
    () => location ?? readBrowserLocation(),
  );
  useEffect(() => {
    if (location) {
      setBrowserLocation(location);
      return undefined;
    }
    if (typeof window === 'undefined') return undefined;
    const update = () => setBrowserLocation(readBrowserLocation());
    window.addEventListener('popstate', update);
    return () => window.removeEventListener('popstate', update);
  }, [location]);
  return browserLocation;
}

function readBrowserLocation(): GithubAuthLocationLike {
  if (typeof window === 'undefined') return { pathname: '/' };
  return {
    pathname: window.location.pathname,
    search: window.location.search,
    hash: window.location.hash,
  };
}
