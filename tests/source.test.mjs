import {test} from 'node:test';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {manifest, source} from '../tools/source.mjs';
test('Rust and JavaScript export the same deployable faction sources',()=>{
 for(const name of manifest.programs){
  const rust=execFileSync('cargo',['run','--quiet','--bin','faction-source','--',name],{encoding:'utf8',maxBuffer:8*1024*1024});
  assert.equal(rust,source(name),name);
  assert.ok(Buffer.byteLength(rust)<=512*1024, `${name} fits the source limit`);
  assert.match(rust,/use gameofcores::\*;/);
  assert.match(rust,/mod shared \{/);
 }
});
test('source lookup rejects unknown names',()=>assert.throws(()=>source('../shared/theme')));
