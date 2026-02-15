const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

function collectSpecFiles(dir, acc = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) collectSpecFiles(p, acc);
    else if (entry.isFile() && entry.name.endsWith('.spec.js')) acc.push(p);
  }
  return acc;
}

const root = process.cwd();
const testsDir = path.join(root, 'tests');
const specs = collectSpecFiles(testsDir).sort();

if (specs.length === 0) {
  console.error('No test spec files found under tests/**/*.spec.js');
  process.exit(1);
}

for (const file of specs) {
  console.log(`\n==> ${path.relative(root, file)}`);
  const r = spawnSync(process.execPath, [file], {
    stdio: 'inherit',
    env: { ...process.env, HARNESS_NOW_SECS: process.env.HARNESS_NOW_SECS || '1800000000' },
  });
  if (r.status !== 0) process.exit(r.status || 1);
}

console.log(`\nAll ${specs.length} test files passed.`);
