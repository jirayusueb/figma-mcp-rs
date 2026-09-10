#!/usr/bin/env node
'use strict';

const { spawnSync } = require('node:child_process');
const { existsSync } = require('node:fs');
const path = require('node:path');

const PLATFORM_MAP = { darwin: 'darwin', linux: 'linux', win32: 'windows' };
const ARCH_MAP = { x64: 'amd64', arm64: 'arm64' };

const osDir = PLATFORM_MAP[process.platform];
const archDir = ARCH_MAP[process.arch];

if (!osDir || !archDir) {
  process.stderr.write(`[figma-mcp-rs] Unsupported platform: ${process.platform}/${process.arch}\n`);
  process.exit(1);
}

const binaryName = process.platform === 'win32' ? 'figma-mcp-rs.exe' : 'figma-mcp-rs';
const binaryPath = path.join(__dirname, `${osDir}-${archDir}`, binaryName);

if (!existsSync(binaryPath)) {
  process.stderr.write(
    '[figma-mcp-rs] Binary not found. Try reinstalling: npm install figma-mcp-rs\n'
  );
  process.exit(1);
}

const result = spawnSync(binaryPath, process.argv.slice(2), { stdio: 'inherit' });
process.exit(result.status ?? 1);
