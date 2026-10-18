// Run authored strategy fixtures against each exported house and the game's actual SDK.
import {mkdtempSync, readFileSync, readdirSync, writeFileSync, mkdirSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import {source} from './source.mjs';

const engine = process.argv[2];
if (!engine) throw new Error('Pass a gameofcores checkout: node tools/test-strategies.mjs ../gameofcores');
const sdk = resolve(engine, 'crates/sdk');
const fixtures = new URL('../tests/strategies/', import.meta.url);
const names = process.argv.slice(3);
const cases = readdirSync(fixtures).filter(f => f.endsWith('.rs') && f !== 'support.rs')
  .filter(f => !names.length || names.includes(f.slice(0, -3))).sort();
if (!cases.length) throw new Error('No strategy fixtures matched.');
const dir = mkdtempSync(join(tmpdir(), 'faction-strategies-'));
try {
  mkdirSync(join(dir, 'src'));
  writeFileSync(join(dir, 'Cargo.toml'), `[package]\nname="faction-strategies"\nversion="0.0.0"\nedition="2021"\n[dependencies]\ngameofcores={path=${JSON.stringify(sdk)}}\n`);
  const support = readFileSync(new URL('support.rs', fixtures), 'utf8');
  for (const file of cases) {
    const name = file.slice(0, -3);
    writeFileSync(join(dir, 'src/lib.rs'), source(name) + '\n#[cfg(test)] mod strategy_tests {\n'
      + support + '\n' + readFileSync(new URL(file, fixtures), 'utf8') + '\n}\n');
    console.log(`Strategy fixtures: ${name}`);
    const run = spawnSync('cargo', ['test', '--manifest-path', join(dir, 'Cargo.toml'), '--lib', '--', '--nocapture'],
      {stdio: 'inherit', env: {...process.env, CARGO_TARGET_DIR: join(dir, 'target')}});
    if (run.error) throw run.error;
    if (run.status !== 0) { process.exitCode = run.status ?? 1; break; }
  }
} finally { rmSync(dir, {recursive: true, force: true}); }
