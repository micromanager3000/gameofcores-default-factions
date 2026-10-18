// Run faction behavior regressions against the SDK in an explicit game checkout.
import {mkdtempSync,readFileSync,readdirSync,writeFileSync,mkdirSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {spawnSync,execFileSync} from 'node:child_process';
import {source} from './source.mjs';
const engine=process.argv[2];
if(!engine)throw new Error('Pass a gameofcores checkout: node tools/test-behavior.mjs ../gameofcores');
const sdk=resolve(engine,'crates/sdk');
const revision=process.argv[3];
let program=source('arjuna');
if(revision){
 const read=path=>execFileSync('git',['show',`${revision}:${path}`],{encoding:'utf8',maxBuffer:2*1024*1024});
 const manifest=JSON.parse(read('manifest.json'));
 program=read('factions/arjuna.rs')+'\n#[allow(dead_code, unused_imports)]\nmod shared {\n'+manifest.shared.map(m=>`pub mod ${m} {\n${read(`shared/${m}.rs`)}\n}\n`).join('')+'}\n';
}
const dir=mkdtempSync(join(tmpdir(),'faction-behavior-'));
try {
 mkdirSync(join(dir,'src'));
 writeFileSync(join(dir,'Cargo.toml'),`[package]\nname="faction-behavior"\nversion="0.0.0"\nedition="2021"\n[dependencies]\ngameofcores={path=${JSON.stringify(sdk)}}\n`);
 const fixtures=new URL('../tests/fixtures/',import.meta.url);
 const tests=['behavior.rs',...readdirSync(fixtures).filter(f=>f.endsWith('_behavior.rs')).sort()];
 writeFileSync(join(dir,'src/lib.rs'),program+'\n#[cfg(test)] mod behavior {\n'+tests.map(f=>readFileSync(new URL(f,fixtures),'utf8')).join('\n')+'\n}\n');
 const run=spawnSync('cargo',['test','--manifest-path',join(dir,'Cargo.toml'),'--','--nocapture'],{stdio:'inherit',env:{...process.env,CARGO_TARGET_DIR:join(dir,'target')}});
 if(run.error)throw run.error;
 process.exitCode=run.status??1;
} finally {rmSync(dir,{recursive:true,force:true});}
