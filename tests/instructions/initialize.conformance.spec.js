const assert = require('assert');
const { invokeInitializeOnProgram, NotImplementedConformanceAdapter } = require('../harness/initialize_adapter');

(async function run(){
  // This is a conformance scaffold: implementation PR must replace adapter and make these assertions real.
  // For spec-lock PR we only enforce scaffold presence and explicit pending state.
  let pending = false;
  try {
    await invokeInitializeOnProgram({});
  } catch (e) {
    if (e instanceof NotImplementedConformanceAdapter) pending = true;
  }
  assert.equal(pending, true, 'conformance adapter must explicitly signal pending until implementation PR');
  console.log('initialize conformance scaffold ok (pending by design)');
})();
