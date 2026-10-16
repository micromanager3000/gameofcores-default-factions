import {readFileSync, writeFileSync, realpathSync, existsSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
const root=new URL('../',import.meta.url);
export const manifest=JSON.parse(readFileSync(new URL('manifest.json',root),'utf8'));
export function source(name){
  const path=manifest.programs.includes(name)?`factions/${name}.rs`:manifest.examples.includes(name)?`examples/structures/${name}.rs`:null;
  if(!path)throw new Error(`Unknown faction: ${name}`);
  const modules=manifest.programs.includes(name)?manifest.shared:manifest.shared.filter(m=>m!=='common'&&m!=='structures'&&m!=='energy');
  // Each house's buildings: a house program carries its own line of shared/buildings.txt, the default program every line.
  const arts=readFileSync(new URL('shared/buildings.txt',root),'utf8').split('\n').filter(l=>l&&!l.startsWith('#')).map(l=>`"${l}"`);
  const seat=manifest.programs.indexOf(name), own=name==='default'?arts.join(', '):arts[seat];
  const art=seat<0?'':`pub fn art(style: usize) -> String { let a = [${own}]; a[style.min(a.len() - 1)].to_string() }\n`;
  const shared='\n#[allow(dead_code, unused_imports)]\nmod shared {\n'+modules.map(m=>`pub mod ${m} {\n${readFileSync(new URL(`shared/${m}.rs`,root),'utf8')}\n}\n`).join('')+art+'}\n';
  return readFileSync(new URL(path,root),'utf8')+shared;
}
if(process.argv[1]&&existsSync(process.argv[1])&&realpathSync(process.argv[1])===fileURLToPath(import.meta.url)){
  const [name,file]=process.argv.slice(2);
  if(file)writeFileSync(file,source(name));else process.stdout.write(source(name));
}
