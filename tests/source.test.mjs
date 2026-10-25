import {test} from 'node:test';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {manifest, source, wears} from '../tools/source.mjs';
test('Rust and JavaScript export the same deployable faction sources',()=>{
 for(const name of [...manifest.programs,...manifest.examples]){
  const rust=execFileSync('cargo',['run','--quiet','--bin','faction-source','--',name],{encoding:'utf8',maxBuffer:8*1024*1024});
  assert.equal(rust,source(name),name);
  assert.ok(Buffer.byteLength(rust)<=512*1024, `${name} fits the source limit`);
  assert.match(rust,/use gameofcores::\*;/);
  assert.match(rust,/mod shared \{/);
  assert.match(rust,/pub enum Role/);
  assert.ok(!rust.includes("default_factions::"), `${name} contains all faction helpers`);
 }
});
test('a house program carries only the styles it wears',()=>{
 assert.match(source('arjuna'),/fn scene10\(/,'Arjuna flies seat 10');
 assert.doesNotMatch(source('arjuna'),/fn scene5\(/,'Arjuna never wears the Reading Room');
 assert.doesNotMatch(source('kublai'),/fn scene10\(/);
 for(let style=0;style<=10;style++){
  assert.match(source('default'),new RegExp(`fn scene${style}\\(`),'the default program wears every style');
  assert.ok(wears('foundry',style),'structure examples wear every style');
 }
});
test('source lookup rejects unknown names',()=>assert.throws(()=>source('../shared/theme')));

test('the source exporter can be imported from stdin scripts',()=>{
 const module=new URL('../tools/source.mjs',import.meta.url).href;
 const output=execFileSync(process.execPath,['--input-type=module','-'],{
  input:`import {source} from ${JSON.stringify(module)};process.stdout.write(source('default'));`,encoding:'utf8',maxBuffer:1024*1024
 });
 assert.equal(output,source('default'));
});
