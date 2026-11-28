/** Planned data removals: which data-removing migrations the production update accepts. */
import assert from 'node:assert/strict';
import { mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { destructiveStatements } from './migration-plan.mjs';
import { contentHash, parsePlannedRemovals, plannedRemoval, readPlannedRemovals } from './planned-removals.mjs';
const DROP = 'ALTER TABLE a DROP COLUMN b;';
const approvals = { '0002_drop.sql': { sha256: contentHash(DROP), reason: 'b moved to table c' } };

test('the content hash is the SHA-256 that sha256sum prints', () => {
  assert.equal(contentHash('abc'), 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
});

test('a listed migration with the approved contents is a planned removal', () => {
  assert.equal(plannedRemoval(approvals, '0002_drop.sql', DROP), 'b moved to table c');
});

test('an edited migration loses its approval', () => {
  assert.equal(plannedRemoval(approvals, '0002_drop.sql', `${DROP} DROP TABLE c;`), undefined);
});

test('a migration that is not listed is not approved, whatever its name looks like', () => {
  assert.equal(plannedRemoval(approvals, '0003_other.sql', DROP), undefined);
  assert.equal(plannedRemoval(approvals, 'constructor', DROP), undefined);
  assert.equal(plannedRemoval({}, '0002_drop.sql', DROP), undefined);
});

test('the list must map migration files to a sha256 and a reason', () => {
  assert.deepEqual(parsePlannedRemovals(JSON.stringify(approvals)), approvals);
  assert.deepEqual(parsePlannedRemovals('{}'), {});
  for (const bad of [
    'not json',
    '[]',
    'null',
    '{"0002_drop.sql": "yes"}',
    '{"0002_drop.sql": {"sha256": "abc", "reason": "short hash"}}',
    `{"0002_drop.sql": {"sha256": "${contentHash(DROP)}"}}`,
    `{"0002_drop.sql": {"sha256": "${contentHash(DROP)}", "reason": " "}}`,
    `{"../0002_drop.sql": {"sha256": "${contentHash(DROP)}", "reason": "not a migration file name"}}`,
  ]) assert.throws(() => parsePlannedRemovals(bad), /planned removals/i, bad);
});

test('a list file is read from disk, and a missing one is an error, not an empty list', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-removals-'));
  try {
    const path = join(dir, 'planned-removals.json');
    writeFileSync(path, JSON.stringify(approvals));
    assert.deepEqual(readPlannedRemovals(path), approvals);
    assert.throws(() => readPlannedRemovals(join(dir, 'absent.json')));
  } finally { rmSync(dir, { recursive: true }); }
});

test('every entry in the repository list names a data-removing migration with those exact contents', () => {
  const dir = new URL('../backend/migrations/', import.meta.url);
  const files = new Set(readdirSync(dir));
  const listed = readPlannedRemovals();
  for (const file of Object.keys(listed)) {
    assert.ok(files.has(file), `${file} is listed but is not a migration`);
    const sql = readFileSync(new URL(file, dir), 'utf8');
    assert.ok(plannedRemoval(listed, file, sql), `${file} does not match its approved sha256`);
    assert.notDeepEqual(destructiveStatements(sql), [], `${file} removes nothing and needs no approval`);
  }
});
