// Regenerates public-only founder structure appearance fixtures for the visual review.
// It compiles a temporary Kublai source because that program carries the full energy module;
// the injected `artfixture` exposes only already-public appearance values.
import {execFileSync} from 'node:child_process';
import {mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {source} from './source.mjs';

const here = new URL('../', import.meta.url), root = new URL('../gameofcores/', here);
const cores = fileURLToPath(new URL('target/release/cores', root)), lines = readFileSync(new URL('shared/buildings.txt', here), 'utf8').split('\n').filter(x => x && !x.startsWith('#'));
const roles = ['landmark', 'hall', 'yard', 'works', 'bastion', 'bay'];
const energy = ['solar', 'boiler', 'wind', 'hydro', 'tidal', 'geo', 'reactor', 'fusion', 'battery'];
const factions = ["Kublai Khan's Alt", "Arjuna's Halting Problem", 'Knights Templated', 'New Atlantis', 'Quiet Fleet', 'The Kernel Manifesto', 'Null Pointer Society', 'Compute of Nations', 'Emu High Command', 'Tabula Rasa'];
const text = new TextEncoder(), decode = new TextDecoder();

function fixtureSource() {
  const all = `pub fn art(style: usize) -> String { let a = [${lines.map(JSON.stringify).join(', ')}]; a[style.min(a.len() - 1)].to_string() }`;
  let out = source('kublai').replace(/pub fn art\(style: usize\) -> String \{[^\n]*\}/, all);
  out = out.replace('program!(think,', 'program!(artfixture, think,');
  return `${out}\nfn artfixture(info: View) -> Value {\n  let style = info.get("style").num() as usize;\n  let role = info.get("role").num_or(-1.0) as isize;\n  if role >= 0 { return shared::structures::architecture(style, shared::structures::ROLES[(role as usize).min(5)]); }\n  shared::energy::look(info.get("kind").str(), info.get("capacity").num_or(8.0), style)\n}\n`;
}
function flat(args) { // the public Cores ABI encoder, kept local so the fixture can run without a built founder module
  let b = new Uint8Array(4096), n = 0; const room = m => { if (n + m > b.length) { const x = new Uint8Array(Math.max(b.length * 2, n + m)); x.set(b); b = x; } };
  const word = x => { room(4); new DataView(b.buffer).setUint32(n, x >>> 0, true); n += 4; }, head = (t, l) => { const at = n; word(t | (Math.min(l, 0x0fffffff) << 4)); return at; }, strings = new Map();
  const str = s => { if (strings.has(s)) return strings.get(s); const at = head(6, text.encode(s).length), q = text.encode(s); room(q.length + 3); b.set(q, n); n += q.length; while (n % 4) b[n++] = 0; strings.set(s, at); return at; };
  const val = v => { if (v === null) return head(1, 0); if (typeof v === 'number') { if (Number.isInteger(v) && v >= -(2 ** 27) && v < 2 ** 27) { const at = n; word(4 | (v << 4)); return at; } const at = head(5, 0); room(8); new DataView(b.buffer).setFloat64(n, v, true); n += 8; return at; } if (typeof v === 'string') return str(v); if (Array.isArray(v)) { const a = v.map(val), at = head(8, a.length); a.forEach(word); return at; } const a = Object.entries(v).map(([k, x]) => [str(k), val(x)]), at = head(9, a.length); a.forEach(([k, x]) => { word(k); word(x); }); return at; };
  room(4 + args.length * 4); n = 4 + args.length * 4; const at = args.map(val), d = new DataView(b.buffer); d.setUint32(0, args.length, true); at.forEach((x, i) => d.setUint32(4 + i * 4, x, true)); return b.slice(0, n);
}
function call(wasm, arg) {
  let out = ''; let instance;
  const imports = {cores:{order(){},log(){},fail(p, n){ out = decode.decode(new Uint8Array(instance.exports.memory.buffer, p, n)); },cycles_left(){ return 1e6; },seed(){ return 7n; },begin(){}}};
  instance = new WebAssembly.Instance(new WebAssembly.Module(wasm), imports); const x = instance.exports, fn = text.encode('artfixture'), a = flat([arg]), at = x.cores_alloc(fn.length + a.length), mem = new Uint8Array(x.memory.buffer); mem.set(fn, at); mem.set(a, at + fn.length);
  if (x.cores_call(at, fn.length, at + fn.length, a.length) !== 0) throw new Error(out || decode.decode(new Uint8Array(x.memory.buffer, x.cores_out_ptr(), x.cores_out_len())));
  return JSON.parse(decode.decode(new Uint8Array(x.memory.buffer, x.cores_out_ptr(), x.cores_out_len())));
}
export function generateStructureArtFixture(destination = new URL('fixtures/structure-art-authored.json', new URL('./', import.meta.url))) {
  const dir = mkdtempSync(join(tmpdir(), 'goc-structure-fixture-')), rs = join(dir, 'fixture.rs'), wasm = join(dir, 'fixture.wasm');
  try {
    const authored = fixtureSource(); writeFileSync(rs, authored); execFileSync(cores, ['compile', rs, wasm], {cwd: root.pathname, stdio: 'pipe'}); const bytes = readFileSync(wasm), appearances = {energy:{}, roles:{}};
    for (let style = 0; style < 10; style++) { appearances.energy[style] = Object.fromEntries(energy.map(kind => [kind, call(bytes, {style, kind, capacity:8})])); appearances.roles[style] = Object.fromEntries(roles.map((role, i) => [role, call(bytes, {style, role:i})])); }
    const value = {version:1, sourceHash:createHash('sha256').update(authored).digest('hex'), factions, energy, roles, appearances}; writeFileSync(destination, JSON.stringify(value)); return value;
  } finally { rmSync(dir, {recursive:true, force:true}); }
}
if (import.meta.url === new URL(process.argv[1], 'file:').href) generateStructureArtFixture();
