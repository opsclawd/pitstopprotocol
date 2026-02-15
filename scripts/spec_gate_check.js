const fs = require('fs');
const path = require('path');

function fail(msg){ console.error(msg); process.exit(1); }

const requiredSpecs = [
  'SPEC_PROTOCOL.md','SPEC_ACCOUNTS.md','SPEC_STATE_MACHINE.md','SPEC_CANONICAL.md','SPEC_INVARIANTS.md','SPEC_ERRORS.md','SPEC_EVENTS.md','SPEC_INSTRUCTIONS/INDEX.md'
];
for (const p of requiredSpecs){ if(!fs.existsSync(p)) fail(`Missing required spec: ${p}`); }

const idx = fs.readFileSync('SPEC_INSTRUCTIONS/INDEX.md','utf8');
const instructionRows = (idx.match(/^\|\s*\d+\s*\|/gm)||[]).length;
if (instructionRows !== 12) fail(`Instruction inventory mismatch: expected 12, found ${instructionRows}`);

// Ensure no TODO in instruction specs when status says LOCKED
const dir='SPEC_INSTRUCTIONS';
for (const f of fs.readdirSync(dir)){
  if(!f.endsWith('.md')||f==='README.md'||f==='INDEX.md') continue;
  const txt=fs.readFileSync(path.join(dir,f),'utf8');
  if (/Status:\s*LOCKED/i.test(txt) && /TODO/.test(txt)) fail(`Locked instruction spec still has TODO: ${f}`);
}

console.log('spec gate check ok');

if (!fs.existsSync("tests/instructions/initialize.spec.js")) fail("Missing required initialize test pack: tests/instructions/initialize.spec.js");
