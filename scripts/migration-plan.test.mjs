/** `make hm-update-server` migration checks: pending files, destructive statements, downgrades. */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { destructiveStatements, directoryPlan, migrationPlan, migrationVersion } from './migration-plan.mjs';
import { contentHash } from './planned-removals.mjs';
import { lookupUser } from './run-with-env.mjs';

test('versions follow sqlx: the numeric prefix, so 0002 and 20261002120000 both parse', () => {
  assert.equal(migrationVersion('0002_notes.sql'), 2n);
  assert.equal(migrationVersion('20261002120000_purchase_history.sql'), 20261002120000n);
  assert.equal(migrationVersion('README.md'), undefined);
});

test('destructive statements are found, but not in comments, strings or index drops', () => {
  assert.deepEqual(destructiveStatements('ALTER TABLE a DROP COLUMN b;'), ['DROP COLUMN']);
  assert.deepEqual(destructiveStatements('alter table a alter column b type text;'), ['column type change']);
  assert.deepEqual(destructiveStatements('DELETE FROM items; TRUNCATE t;'), ['TRUNCATE', 'DELETE FROM']);
  assert.deepEqual(destructiveStatements("-- DROP TABLE x\n/* TRUNCATE y */ INSERT INTO n VALUES ('DROP TABLE z');"), []);
  assert.deepEqual(destructiveStatements('DROP INDEX IF EXISTS i; ALTER TABLE a DROP CONSTRAINT c;'), []);
  assert.deepEqual(destructiveStatements('DO $$ BEGIN RAISE NOTICE \'DELETE FROM\'; END $$;'), []);
});

test('only unapplied files are pending, in version order', () => {
  const files = ['0003_c.sql', '0001_a.sql', '0002_b.sql'];
  const plan = migrationPlan(files, ['1'], () => 'CREATE TABLE t (id int);');
  assert.deepEqual(plan.pending.map(({ file }) => file), ['0002_b.sql', '0003_c.sql']);
  assert.deepEqual(plan.ahead, []);
});

test('a database with migrations the checkout lacks is reported as ahead', () => {
  assert.deepEqual(migrationPlan(['0001_a.sql'], ['1', '2'], () => '').ahead, ['2']);
});

test('a planned removal is reported apart from a destructive statement, and only for its exact contents', () => {
  const drop = 'ALTER TABLE a DROP COLUMN b;';
  const approvals = { '0002_b.sql': { sha256: contentHash(drop), reason: 'b moved' } };
  const plan = sql => migrationPlan(['0001_a.sql', '0002_b.sql'], ['1'], () => sql, approvals).pending;
  assert.deepEqual(plan(drop), [{ file: '0002_b.sql', destructive: [], planned: ['DROP COLUMN'], reason: 'b moved' }]);
  assert.deepEqual(plan(`${drop} TRUNCATE a;`), [{ file: '0002_b.sql', destructive: ['DROP COLUMN', 'TRUNCATE'], planned: [] }]);
  assert.deepEqual(migrationPlan(['0002_b.sql'], [], () => drop).pending[0].destructive, ['DROP COLUMN']);
});

test('every data-removing repository migration is a planned removal', () => {
  const plan = directoryPlan(fileURLToPath(new URL('../backend/migrations/', import.meta.url)), []);
  assert.ok(plan.pending.length > 0);
  for (const { file, destructive } of plan.pending) assert.deepEqual(destructive, [], file);
  assert.deepEqual(plan.pending.filter(({ planned }) => planned.length).map(({ file }) => file),
    ['20261128090000_drop_purchase_category.sql']);
});

test('the command exits 3 for a destructive pending migration, 4 for a downgrade and 0 for a planned removal', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grocery-migrations-'));
  try {
    writeFileSync(join(dir, '0001_a.sql'), 'CREATE TABLE a (id int);');
    writeFileSync(join(dir, '0002_b.sql'), 'DROP TABLE a;');
    const run = input => spawnSync(process.execPath, ['scripts/migration-plan.mjs', dir], { input, encoding: 'utf8' });
    assert.equal(run('1\n2\n').status, 0);
    const destructive = run('1\n');
    assert.equal(destructive.status, 3);
    assert.match(destructive.stdout, /destructive 0002_b.sql: DROP TABLE/);
    assert.equal(run('').status, 3);
    assert.equal(run('1\n2\n3\n').status, 4);
    const approvals = join(dir, 'approvals.json');
    writeFileSync(approvals, JSON.stringify({ '0002_b.sql': { sha256: contentHash('DROP TABLE a;'), reason: 'a is unused' } }));
    const planned = spawnSync(process.execPath, ['scripts/migration-plan.mjs', dir, approvals], { input: '1\n', encoding: 'utf8' });
    assert.equal(planned.status, 0);
    assert.match(planned.stdout, /planned removal 0002_b.sql: DROP TABLE \(a is unused\)/);
    writeFileSync(approvals, 'not json');
    assert.equal(spawnSync(process.execPath, ['scripts/migration-plan.mjs', dir, approvals], { input: '1\n' }).status, 1);
  } finally { rmSync(dir, { recursive: true }); }
});

test('service users are looked up from passwd', () => {
  assert.deepEqual(lookupUser('nobody', 'root:x:0:0::/root:/bin/sh\nnobody:x:65534:65534::/:/usr/sbin/nologin\n'), { uid: 65534, gid: 65534 });
  assert.throws(() => lookupUser('ghost', ''), /No such user/);
});
