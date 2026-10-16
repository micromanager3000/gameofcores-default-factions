import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {manifest, source as exported} from '../tools/source.mjs';

const root=new URL('../examples/structures/',import.meta.url);
const provenance=JSON.parse(readFileSync(new URL('provenance.json',root),'utf8'));
test('structure examples preserve measured strategy and authored geometry',()=>{
 assert.deepEqual(manifest.examples,provenance.sources.map(s=>s.name));
 assert.match(provenance.revision,/^[a-f0-9]{40}$/);
 for(const record of provenance.sources){
  const source=readFileSync(new URL(`${record.name}.rs`,root),'utf8');
  const original=source
   .replace(/use gameofcores::\*;\nuse crate::shared::(?:\{capital, theme\}|capital);/,'use gameofcores::*;')
   .replace(/    use gameofcores::\*;\n    use crate::shared::(?:\{air, theme\}|theme);/,'    use gameofcores::*;')
   .replace(/capital::(civic|Civic|ring|spare)\b/g,'build3d::$1');
  assert.equal(createHash('sha256').update(original).digest('hex'),record.sha256,record.name);
  assert.equal((exported(record.name).match(/pub mod common \{/g)||[]).length,1,`${record.name} carries one strategy helper module`);
 }
});
