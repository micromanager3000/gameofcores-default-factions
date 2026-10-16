// Run faction behavior regressions against the SDK in an explicit game checkout.
import {mkdtempSync,readFileSync,writeFileSync,mkdirSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import {source} from './source.mjs';
const engine=process.argv[2];
if(!engine)throw new Error('Pass a gameofcores checkout: node tools/test-behavior.mjs ../gameofcores');
const sdk=resolve(engine,'crates/sdk');
const dir=mkdtempSync(join(tmpdir(),'faction-behavior-'));
try {
 mkdirSync(join(dir,'src'));
 writeFileSync(join(dir,'Cargo.toml'),`[package]\nname="faction-behavior"\nversion="0.0.0"\nedition="2021"\n[dependencies]\ngameofcores={path=${JSON.stringify(sdk)}}\n`);
 writeFileSync(join(dir,'src/lib.rs'),source('arjuna')+'\n#[cfg(test)] mod behavior {\n'+readFileSync(new URL('../tests/fixtures/behavior.rs',import.meta.url),'utf8')+'\n}\n');
 const run=spawnSync('cargo',['test','--manifest-path',join(dir,'Cargo.toml'),'--','--nocapture'],{stdio:'inherit',env:{...process.env,CARGO_TARGET_DIR:join(dir,'target')}});
 if(run.error)throw run.error;
 process.exitCode=run.status??1;
} finally {rmSync(dir,{recursive:true,force:true});}
