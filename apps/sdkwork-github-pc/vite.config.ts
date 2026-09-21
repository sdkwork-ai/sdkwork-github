import { resolveViteEnvironment, resolveLucideReactEntry } from '../../../sdkwork-specs/tools/vite-runtime-profile.mjs';
import { resolveBrowserDistOutDir } from '../../../sdkwork-specs/tools/browser-dist-layout.mjs';

import { defineConfig, loadEnv } from 'vite';
import { createSdkworkCredentialEntryBootstrapVitePlugin } from '@sdkwork/iam-credential-entry/vite';
import react from '@vitejs/plugin-react';

const defaultApiTarget = 'http://127.0.0.1:4100';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, '.', '');
  const bootstrapAccessToken = env.SDKWORK_ACCESS_TOKEN ?? process.env.SDKWORK_ACCESS_TOKEN;
  const apiTarget =
    env.VITE_SDKWORK_GITHUB_APPLICATION_PUBLIC_HTTP_URL
    ?? process.env.VITE_SDKWORK_GITHUB_APPLICATION_PUBLIC_HTTP_URL
    ?? defaultApiTarget;

  return {
    build: {
      outDir: resolveBrowserDistOutDir(resolveViteEnvironment(mode, process.env)),
      emptyOutDir: true,
    },
    plugins: [
      // The bootstrap credential reaches the renderer only through the shared IAM
      // plugin (dev-server HTML injection as
      // `globalThis.__SDKWORK_CREDENTIAL_ENTRY_BOOTSTRAP_ACCESS_TOKEN__`).
      // `define['process.env.SDKWORK_ACCESS_TOKEN']` is NOT a valid handoff
      // (IAM_CREDENTIAL_ENTRY_SPEC.md section 4/5).
      createSdkworkCredentialEntryBootstrapVitePlugin({
        accessToken: bootstrapAccessToken,
        environment: resolveViteEnvironment(mode, process.env),
      }),
      react(),
    ],
    server: {
      port: 5175,
      proxy: {
        '/app/v3/api': {
          target: apiTarget,
          changeOrigin: true,
        },
      },
    },
  };
});
