// Build the faction-owned dependency beside the SDK built by the same Rust toolchain.
import {execFileSync} from 'node:child_process';
import {existsSync, renameSync, rmSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {fileURLToPath} from 'node:url';
const sdk=resolve(process.argv[2]||'../gameofcores/target/wasm32-unknown-unknown/release');
const library=fileURLToPath(new URL('../library.rs',import.meta.url));
if(!existsSync(join(sdk,'libgameofcores.rlib')))throw new Error('Build the game SDK for wasm32-unknown-unknown first.');
const output=join(sdk,'libdefault_factions.rlib'),temp=output+'.next';
try{
 execFileSync('rustc',['--edition=2021','--crate-type=rlib','--crate-name=default_factions',
  '--target=wasm32-unknown-unknown','-C','opt-level=3','-C','embed-bitcode=yes','--cap-lints=allow',
  '--extern',`gameofcores=${join(sdk,'libgameofcores.rlib')}`,'-L',`dependency=${join(sdk,'deps')}`,
  '-o',temp,library],{stdio:'inherit'});
 renameSync(temp,output);
 console.log(`Built ${output}`);
}finally{rmSync(temp,{force:true});}
