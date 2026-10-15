import {readFileSync, writeFileSync, realpathSync, existsSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
const root=new URL('../',import.meta.url);
export const manifest=JSON.parse(readFileSync(new URL('manifest.json',root),'utf8'));
export function source(name){
  if(!manifest.programs.includes(name))throw new Error(`Unknown faction: ${name}`);
  const shared='\n#[allow(dead_code, unused_imports)]\nmod shared {\n'+manifest.shared.map(m=>`pub mod ${m} {\n${readFileSync(new URL(`shared/${m}.rs`,root),'utf8')}\n}\n`).join('')+'}\n';
  return readFileSync(new URL(`factions/${name}.rs`,root),'utf8')+shared;
}
if(process.argv[1]&&existsSync(process.argv[1])&&realpathSync(process.argv[1])===fileURLToPath(import.meta.url)){
  const [name,file]=process.argv.slice(2);
  if(file)writeFileSync(file,source(name));else process.stdout.write(source(name));
}
