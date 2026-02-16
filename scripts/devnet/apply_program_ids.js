#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

const repoRoot = path.resolve(__dirname, '..', '..');
const idsPath = path.join(repoRoot, 'configs', 'program-ids.json');
const anchorTomlPath = path.join(repoRoot, 'Anchor.toml');
const libPath = path.join(repoRoot, 'programs', 'pitstop', 'src', 'lib.rs');

const checkOnly = process.argv.includes('--check');

const ids = JSON.parse(fs.readFileSync(idsPath, 'utf8'));
const programId = ids?.pitstop?.devnet;
if (!programId || programId.includes('TODO')) {
  throw new Error('configs/program-ids.json pitstop.devnet is missing or TODO');
}

const updates = [];

let anchorToml = fs.readFileSync(anchorTomlPath, 'utf8');
const anchorRe = /(\[programs\.devnet\][\s\S]*?pitstop\s*=\s*")([^"]+)(")/;
if (!anchorRe.test(anchorToml)) {
  throw new Error('Could not find [programs.devnet] pitstop entry in Anchor.toml');
}
anchorToml = anchorToml.replace(anchorRe, `$1${programId}$3`);
updates.push(['Anchor.toml', anchorTomlPath, anchorToml]);

let libRs = fs.readFileSync(libPath, 'utf8');
const declareRe = /(declare_id!\(")([^"]+)("\);)/;
if (!declareRe.test(libRs)) {
  throw new Error('Could not find declare_id!(...) in programs/pitstop/src/lib.rs');
}
libRs = libRs.replace(declareRe, `$1${programId}$3`);
updates.push(['programs/pitstop/src/lib.rs', libPath, libRs]);

if (checkOnly) {
  let failed = false;
  for (const [name, filePath, next] of updates) {
    const current = fs.readFileSync(filePath, 'utf8');
    if (current !== next) {
      failed = true;
      console.error(`Mismatch: ${name} is not synced to configs/program-ids.json`);
    }
  }
  if (failed) process.exit(1);
  console.log('program id sync check ok');
  process.exit(0);
}

for (const [, filePath, next] of updates) {
  fs.writeFileSync(filePath, next);
}
console.log(`Applied pitstop devnet program id: ${programId}`);
