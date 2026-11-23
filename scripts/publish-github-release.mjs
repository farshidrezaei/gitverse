#!/usr/bin/env node

/**
 * GitVerse GitHub Release Publisher
 *
 * Extracts release notes from CHANGELOG.md, appends official desktop
 * installer matrix and checksum information, and publishes the draft GitHub release.
 */

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

export function extractChangelogSection(changelogContent, version) {
  const versionHeader = `## [${version}]`;
  const lines = changelogContent.split('\n');
  const sectionLines = [];
  let capturing = false;

  for (const line of lines) {
    if (line.startsWith(versionHeader)) {
      capturing = true;
      continue;
    }
    if (capturing && line.startsWith('## [')) {
      break;
    }
    if (capturing) {
      sectionLines.push(line);
    }
  }

  return sectionLines.join('\n').trim();
}

export function buildReleaseBody(version, notes) {
  const binaryTable = [
    '### Official Desktop Installers',
    '',
    '| Platform | Architecture | Package Format |',
    '|---|---|---|',
    '| **macOS** | Universal (`Apple Silicon` + `Intel`) | `.dmg` |',
    '| **Linux** | `x86_64` | `.AppImage` (Portable) |',
    '| **Linux** | `x86_64` | `.deb` (Debian / Ubuntu) |',
    '| **Linux** | `x86_64` | `.rpm` (Fedora / RHEL / openSUSE) |',
    '| **Windows** | `x64` | `.exe` (NSIS Installer) |',
    '| **Windows** | `x64` | `.msi` (Enterprise MSI) |',
    '',
    '*SHA256 checksums are verified and recorded in `SHA256SUMS.txt` attached below.*',
  ].join('\n');

  if (notes) {
    return `## What is Changed in GitVerse v${version}\n\n${notes}\n\n---\n\n${binaryTable}`;
  }
  return binaryTable;
}

async function run() {
  const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN;
  const releaseId = process.env.RELEASE_ID;
  const repository = process.env.GITHUB_REPOSITORY || 'farshidrezaei/gitverse';

  if (!token) {
    throw new Error('GITHUB_TOKEN or GH_TOKEN environment variable is required');
  }
  if (!releaseId) {
    throw new Error('RELEASE_ID environment variable is required');
  }

  const tauriConfigPath = resolve(process.cwd(), 'src-tauri/tauri.conf.json');
  const tauriConfig = JSON.parse(readFileSync(tauriConfigPath, 'utf8'));
  const version = tauriConfig.version;

  const changelogPath = resolve(process.cwd(), 'CHANGELOG.md');
  const changelog = readFileSync(changelogPath, 'utf8');
  const notes = extractChangelogSection(changelog, version);
  const body = buildReleaseBody(version, notes);

  console.log(`Publishing draft release ${releaseId} for version ${version} on ${repository}...`);

  const response = await fetch(`https://api.github.com/repos/${repository}/releases/${releaseId}`, {
    method: 'PATCH',
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: 'application/vnd.github+json',
      'Content-Type': 'application/json',
      'User-Agent': 'GitVerse-Release-Bot',
    },
    body: JSON.stringify({
      draft: false,
      body,
    }),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Failed to publish release: ${response.status} ${response.statusText} - ${errorText}`);
  }

  const data = await response.json();
  console.log(`Successfully published release v${version} (ID: ${data.id}) - ${data.html_url}`);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  run().catch((error) => {
    console.error(error);
    process.exit(1);
  });
}
