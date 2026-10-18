import {test} from 'node:test';
import assert from 'node:assert/strict';
import {source} from '../tools/source.mjs';

test('the default export carries the active grid planner and transmission research', () => {
  const program = source('default');
  assert.match(program, /const TECHS: \[&str; 15\].*"transmission"/);
  assert.match(program, /pub fn apply\(orders: &mut Value, world: View, me: View\)/);
  assert.match(program, /fn sell\(orders: &mut Value, world: View, me: View,/);
  assert.match(program, /pub fn export_spare\(me: View, buyer: f64\) -> f64/);
  assert.doesNotMatch(program, /Plans nothing: the house generator/);
  assert.ok(Buffer.byteLength(program) <= 512 * 1024, 'default source fits the game source limit');
});
