const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

function fail(msg){ console.error(msg); process.exit(1); }

const requiredSpecs = [
  'SPEC_PROTOCOL.md','SPEC_ACCOUNTS.md','SPEC_STATE_MACHINE.md','SPEC_CANONICAL.md','SPEC_INVARIANTS.md','SPEC_ERRORS.md','SPEC_EVENTS.md','SPEC_INSTRUCTIONS/INDEX.md'
];
for (const p of requiredSpecs){ if(!fs.existsSync(p)) fail(`Missing required spec: ${p}`); }

const idx = fs.readFileSync('SPEC_INSTRUCTIONS/INDEX.md','utf8');
const instructionRows = (idx.match(/^\|\s*\d+\s*\|/gm)||[]).length;
if (instructionRows !== 12) fail(`Instruction inventory mismatch: expected 12, found ${instructionRows}`);

for (const f of fs.readdirSync('SPEC_INSTRUCTIONS')){
  if(!f.endsWith('.md')||f==='README.md'||f==='INDEX.md') continue;
  const txt=fs.readFileSync(path.join('SPEC_INSTRUCTIONS',f),'utf8');
  if (/Status:\s*LOCKED/i.test(txt) && /TODO/.test(txt)) fail(`Locked instruction spec still has TODO: ${f}`);
}

if (!fs.existsSync('tests/instructions/initialize.spec.js')) fail('Missing required initialize test pack: tests/instructions/initialize.spec.js');
if (!fs.existsSync('tests/instructions/initialize.conformance.spec.js')) fail('Missing required initialize conformance scaffold: tests/instructions/initialize.conformance.spec.js');

let changed=[];
try {
  changed = execSync('git diff --name-only origin/main...HEAD', {encoding:'utf8'}).split('\n').filter(Boolean);
} catch {}

const lockedSpecPaths = changed.filter(p => /^SPEC_INSTRUCTIONS\/.+\.md$/.test(p) || ['SPEC_PROTOCOL.md','SPEC_CANONICAL.md','SPEC_INVARIANTS.md','SPEC_ERRORS.md','SPEC_EVENTS.md','SPEC_STATE_MACHINE.md','SPEC_ACCOUNTS.md'].includes(p));
for (const p of lockedSpecPaths) {
  if (!fs.existsSync(p)) continue;
  const cur = fs.readFileSync(p,'utf8');
  const curLocked = /Status:\s*LOCKED/i.test(cur);
  if (!curLocked) continue;
  let prev='';
  try { prev = execSync(`git show origin/main:${p}`, {encoding:'utf8'}); } catch { continue; }
  const prevVer = (prev.match(/Version:\s*(v[0-9]+\.[0-9]+\.[0-9]+)/i)||[])[1];
  const curVer = (cur.match(/Version:\s*(v[0-9]+\.[0-9]+\.[0-9]+)/i)||[])[1];
  if (prevVer && curVer && prevVer === curVer && cur !== prev) fail(`Locked spec changed without version bump: ${p} (${curVer})`);
}

{
  const txt = fs.readFileSync('SPEC_ERRORS.md','utf8');
  if (!/Status:\s*LOCKED/i.test(txt)) fail('SPEC_ERRORS.md must be LOCKED');
  if (!/## Instruction mapping/i.test(txt)) fail('SPEC_ERRORS.md missing instruction mapping section');
}
{
  const txt = fs.readFileSync('SPEC_EVENTS.md','utf8');
  if (!/Status:\s*LOCKED/i.test(txt)) fail('SPEC_EVENTS.md must be LOCKED');
  if (!/must-emit matrix/i.test(txt)) fail('SPEC_EVENTS.md missing must-emit matrix section');
}
{
  const txt = fs.readFileSync('SPEC_STATE_MACHINE.md','utf8');
  if (!/Status:\s*LOCKED/i.test(txt)) fail('SPEC_STATE_MACHINE.md must be LOCKED');
}
{
  const eventsTxt = fs.readFileSync('SPEC_EVENTS.md', 'utf8');
  const known = new Set();
  for (const m of eventsTxt.matchAll(/^-\s+([A-Za-z][A-Za-z0-9_]*)\s+\{/gm)) known.add(m[1]);
  const refs = [];
  for (const f of fs.readdirSync('SPEC_INSTRUCTIONS')) {
    if (!f.endsWith('.md') || f === 'README.md' || f === 'INDEX.md') continue;
    const txt = fs.readFileSync(path.join('SPEC_INSTRUCTIONS', f), 'utf8');
    const section = txt.match(/## Events[\s\S]*?(?=\n## |$)/m);
    if (!section) continue;
    for (const m of section[0].matchAll(/^\-\s+`([A-Za-z][A-Za-z0-9_]*)`/gm)) refs.push({ file: f, event: m[1] });
  }
  const bad = refs.filter(r => !known.has(r.event));
  if (bad.length) fail('unknown event(s) referenced by instruction specs: ' + bad.map(b => `${b.file}:${b.event}`).join(', '));
}

console.log('spec gate check ok');
