#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const OFFICIAL_LANGUAGE_ORDER = ['typescript', 'rust', 'java', 'python', 'go'];
const DEFAULT_LANGUAGE = 'typescript';
const FIXED_SDK_VERSION = '0.1.0';
const STANDARD_PROFILE = 'sdkwork-v3';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, '..');
const STANDARD_SDK_GENERATOR_BIN = path.resolve(
  workspaceRoot,
  '../sdkwork-sdk-generator/bin/sdkgen.js',
);

function fail(sdkName, message) {
  process.stderr.write(`[${sdkName}] ${message}\n`);
  process.exit(1);
}

function parseLanguages(raw, sdkName) {
  const requested = raw.flatMap((value) => String(value || '').split(','));
  const normalized = [];
  for (const item of requested) {
    const language = item.trim().toLowerCase();
    if (!language) continue;
    if (!OFFICIAL_LANGUAGE_ORDER.includes(language)) {
      fail(sdkName, `unsupported language: ${language}`);
    }
    if (!normalized.includes(language)) normalized.push(language);
  }
  return OFFICIAL_LANGUAGE_ORDER.filter((language) => normalized.includes(language));
}

function parseArgs(argv, defaultBaseUrl, sdkName) {
  const parsed = {
    allLanguages: false,
    baseUrl: defaultBaseUrl,
    input: null,
    languages: [],
    passthrough: [],
  };

  for (let index = 0; index < argv.length; index += 1) {
    const current = argv[index];
    if (current === '--all-languages') {
      parsed.allLanguages = true;
      continue;
    }
    if (current === '--language') {
      parsed.languages.push(argv[index + 1] || '');
      index += 1;
      continue;
    }
    if (current.startsWith('--language=')) {
      parsed.languages.push(current.slice('--language='.length));
      continue;
    }
    if (current === '--base-url') {
      parsed.baseUrl = argv[index + 1] || defaultBaseUrl;
      index += 1;
      continue;
    }
    if (current === '--input') {
      parsed.input = argv[index + 1] || '';
      index += 1;
      continue;
    }
    if (current.startsWith('--input=')) {
      parsed.input = current.slice('--input='.length);
      continue;
    }
    if (current === '--') {
      parsed.passthrough.push(...argv.slice(index + 1));
      break;
    }
    parsed.passthrough.push(current);
  }

  if (!parsed.baseUrl.trim()) {
    fail(sdkName, 'base URL cannot be empty');
  }
  return parsed;
}

function operations(document) {
  const methods = new Set(['get', 'post', 'put', 'patch', 'delete']);
  const result = [];
  for (const [pathKey, pathItem] of Object.entries(document.paths || {})) {
    for (const [methodName, operation] of Object.entries(pathItem || {})) {
      if (!methods.has(methodName)) continue;
      result.push({
        method: methodName.toUpperCase(),
        operationId: operation.operationId,
        path: pathKey,
      });
    }
  }
  return result.sort((left, right) => left.operationId.localeCompare(right.operationId));
}

function writeSdkManifest({ family, inputPath, baseUrl, languages }) {
  const document = JSON.parse(readFileSync(inputPath, 'utf8'));
  const manifestPath = path.join(family.sdkRoot, 'sdk-manifest.json');
  const currentManifest = existsSync(manifestPath)
    ? JSON.parse(readFileSync(manifestPath, 'utf8'))
    : {};
  const manifest = {
    ...currentManifest,
    schemaVersion: 1,
    sdkName: family.sdkName,
    sdkOwner: 'sdkwork-github',
    apiAuthority: family.apiAuthority,
    sdkFamily: family.sdkName,
    sdkType: family.sdkType,
    apiPrefix: family.apiPrefix,
    generationInputSpec: toPosix(path.relative(family.sdkRoot, inputPath)),
    generatedPackages: Object.fromEntries(
      languages.map((language) => [
        language,
        {
          language,
          packageName: `${family.sdkName}-generated-${language}`,
          generatedOutput: `${family.sdkName}-${language}/generated/server-openapi`,
        },
      ]),
    ),
    generatorName: '@sdkwork/sdk-generator',
    generatorPath: STANDARD_SDK_GENERATOR_BIN,
    baseUrl,
    standardProfile: STANDARD_PROFILE,
    fixedSdkVersion: FIXED_SDK_VERSION,
    ownerOnlyOperationCount: operations(document).length,
    operations: operations(document),
    sdkDependencies: [
      {
        workspace: 'sdkwork-iam-app-sdk',
        role: 'appbase-app-capability',
        required: true,
        dependencyMode: 'consumer-sdk',
        apiPrefix: '/app/v3/api',
        apiAuthority: 'sdkwork-iam-app-api',
        generatedTransportImportPolicy: 'forbidden',
        packageByLanguage: {
          typescript: '@sdkwork/iam-app-sdk',
        },
      },
    ],
    dependencyApiExports: [],
    managedBy: 'tools/github_sdk_generator_runner.mjs',
  };
  writeFileSync(
    path.join(family.sdkRoot, 'sdk-manifest.json'),
    `${JSON.stringify(manifest, null, 2)}\n`,
    'utf8',
  );
}

function syncAssembly(family, inputPath) {
  const assemblyPath = path.join(family.sdkRoot, 'sdk-manifest.json');
  const assembly = existsSync(assemblyPath)
    ? JSON.parse(readFileSync(assemblyPath, 'utf8'))
    : {};
  const relativeInput = toPosix(path.relative(family.sdkRoot, inputPath));
  assembly.workspace = family.sdkName;
  assembly.sdkOwner = 'sdkwork-github';
  assembly.apiAuthority = family.apiAuthority;
  assembly.authoritySpec = relativeInput;
  assembly.generationInputSpec = relativeInput;
  assembly.derivedSpecs = { ...(assembly.derivedSpecs || {}), default: relativeInput };
  assembly.discoverySurface = {
    ...(assembly.discoverySurface || {}),
    sdkTarget: family.sdkType,
    apiPrefix: family.apiPrefix,
    generatedProtocols: ['http-openapi'],
    manualTransports: [],
  };
  writeFileSync(assemblyPath, `${JSON.stringify(assembly, null, 2)}\n`, 'utf8');
}

function toPosix(value) {
  return value.replace(/\\/g, '/');
}

export function resolveFamilySdkRoot(importMetaUrl) {
  return path.resolve(path.dirname(fileURLToPath(importMetaUrl)), '..');
}

export function runGithubSdkGenerator(family, argv) {
  if (!existsSync(STANDARD_SDK_GENERATOR_BIN)) {
    fail(family.sdkName, `standard SDK generator not found: ${STANDARD_SDK_GENERATOR_BIN}`);
  }

  const args = parseArgs(argv, family.defaultBaseUrl, family.sdkName);
  const inputPath = args.input
    ? path.resolve(workspaceRoot, args.input)
    : path.join(workspaceRoot, 'apis/app-api/github/github-app-api.openapi.json');
  if (!existsSync(inputPath)) {
    fail(family.sdkName, `OpenAPI input not found: ${inputPath}`);
  }

  const languages = args.allLanguages
    ? OFFICIAL_LANGUAGE_ORDER
    : parseLanguages(args.languages.length ? args.languages : [DEFAULT_LANGUAGE], family.sdkName);

  mkdirSync(family.sdkRoot, { recursive: true });
  syncAssembly(family, inputPath);

  for (const language of languages) {
    const outputPath = path.join(
      family.sdkRoot,
      `${family.sdkName}-${language}`,
      'generated',
      'server-openapi',
    );
    const generatorArgs = [
      STANDARD_SDK_GENERATOR_BIN,
      'generate',
      '--input',
      inputPath,
      '--output',
      outputPath,
      '--name',
      family.sdkName,
      '--type',
      family.sdkType,
      '--language',
      language,
      '--base-url',
      args.baseUrl,
      '--api-prefix',
      family.apiPrefix,
      '--package-name',
      `${family.sdkName}-generated-${language}`,
      '--fixed-sdk-version',
      FIXED_SDK_VERSION,
      '--sdk-root',
      family.sdkRoot,
      '--sdk-name',
      family.sdkName,
      '--standard-profile',
      STANDARD_PROFILE,
      ...args.passthrough,
    ];
    const result = spawnSync('node', generatorArgs, {
      cwd: family.sdkRoot,
      stdio: 'inherit',
    });
    if (result.status !== 0) {
      fail(family.sdkName, `sdkgen failed for ${language} with exit code ${result.status}`);
    }
  }

  writeSdkManifest({ baseUrl: args.baseUrl, family, inputPath, languages });
}
