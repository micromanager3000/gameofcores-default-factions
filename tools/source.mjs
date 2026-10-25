import {readFileSync, writeFileSync, realpathSync, existsSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
const root=new URL('../',import.meta.url);
export const manifest=JSON.parse(readFileSync(new URL('manifest.json',root),'utf8'));
// Whether program `name` wears founding style `style` (a seat of shared/theme.rs): the seats it founds and its home
// style. A house program leaves the other styles out (a program pays for its length); the default program and the
// structure examples wear every style. (build.rs does the same.)
const WEARS={kublai:[0],arjuna:[0,1,8,10],templar:[2],atlantis:[0,3,9],fleet:[0,4],manifesto:[0,5],nullptr:[0,6],nations:[0,7]};
export const wears=(name,style)=>!WEARS[name]||WEARS[name].includes(style);
// Shared code for program `name`: the default program leaves out code fenced `// @house` … `// @end`, but carries the
// same grid planner as the founders so unclaimed factions can build, store and trade power; a program that doesn't wear
// style N leaves out code fenced `// @style N` … `// @end`, and writes the rest of an `// @else` line in it in its place
// (a comment otherwise, so the file compiles as it stands, wearing every style).
export function fence(text,name){
  let drop=false;const out=[];
  for(const line of text.split('\n')){
    const t=line.trimStart();
    if(t.startsWith('// @house'))drop=name==='default';
    else if(t.startsWith('// @style ')){const n=Number(t.slice(10).trim().split(/\s+/)[0]);if(!Number.isInteger(n))throw new Error(`Not a style number: ${line}`);drop=!wears(name,n);}
    else if(t.startsWith('// @else ')){if(drop)out.push(line.slice(0,line.length-t.length)+t.slice(9));continue;}
    else if(t.startsWith('// @end')){if(!drop)out.push(line);drop=false;continue;}
    if(!drop)out.push(line);
  }
  return out.join('\n');
}
export function source(name){
  const path=manifest.programs.includes(name)?`factions/${name}.rs`:manifest.examples.includes(name)?`examples/structures/${name}.rs`:null;
  if(!path)throw new Error(`Unknown faction: ${name}`);
  const modules=manifest.programs.includes(name)?manifest.shared:manifest.shared.filter(m=>m!=='common'&&m!=='structures'&&m!=='energy');
  // Each house's buildings: a house program carries its own line of shared/buildings.txt (Arjuna's also seat 10's, since
  // it flies for Highway to the Danger Node there), the default program every line.
  const arts=readFileSync(new URL('shared/buildings.txt',root),'utf8').split('\n').filter(l=>l&&!l.startsWith('#')).map(l=>`"${l}"`);
  const seat=manifest.programs.indexOf(name);
  const [own,pick]=name==='default'?[arts.join(', '),'style.min(a.len() - 1)']:name==='arjuna'?[`${arts[seat]}, ${arts[10]}`,'usize::from(style == 10)']:[arts[seat],'style.min(a.len() - 1)'];
  const art=seat<0?'':`pub fn art(style: usize) -> String { let a = [${own}]; a[${pick}].to_string() }\n`;
  const shared='\n#[allow(dead_code, unused_imports)]\nmod shared {\n'+modules.map(m=>`pub mod ${m} {\n${fence(readFileSync(new URL(`shared/${m}.rs`,root),'utf8'),name)}\n}\n`).join('')+art+'}\n';
  return readFileSync(new URL(path,root),'utf8')+shared;
}
if(process.argv[1]&&existsSync(process.argv[1])&&realpathSync(process.argv[1])===fileURLToPath(import.meta.url)){
  const [name,file]=process.argv.slice(2);
  if(file)writeFileSync(file,source(name));else process.stdout.write(source(name));
}
