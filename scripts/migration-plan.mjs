/**
 * Decide whether `make hm-update-server` may migrate the production database.
 *
 *   psql -Atc 'select version from _sqlx_migrations where success' \
 *     | node migration-plan.mjs MIGRATIONS_DIR [PLANNED_REMOVALS_FILE]
 *
 * Prints one line per pending migration and per destructive statement it
 * contains. Exit status: 0 safe to apply, 3 a pending migration may destroy
 * data, 4 the database is ahead of this checkout (an older branch).
 *
 * A data-removing migration is safe to apply only as a planned removal: one
 * listed, with its exact contents, in `planned-removals.json` (see
 * `planned-removals.mjs`).
 *
 * sqlx itself applies only pending files, in order, each in a transaction,
 * and refuses to run if an applied file was edited. This adds the checks it
 * does not make: data-destroying statements and a downgrade.
 */
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { plannedRemoval, readPlannedRemovals } from './planned-removals.mjs';

/** Statements that remove or rewrite stored data. Dropping an index or constraint does not. */
const DESTRUCTIVE = [
  ['DROP TABLE', /\bDROP\s+TABLE\b/i],
  ['DROP COLUMN', /\bDROP\s+COLUMN\b/i],
  ['DROP SCHEMA', /\bDROP\s+SCHEMA\b/i],
  ['DROP DATABASE', /\bDROP\s+DATABASE\b/i],
  ['TRUNCATE', /\bTRUNCATE\b/i],
  ['DELETE FROM', /\bDELETE\s+FROM\b/i],
  ['column type change', /\bALTER\s+COLUMN\s+\S+\s+(SET\s+DATA\s+)?TYPE\b/i],
];

/** The version sqlx records: the file name's numeric prefix. */
export function migrationVersion(file) {
  const match = /^(\d+)_.*\.sql$/.exec(file);
  return match ? BigInt(match[1]) : undefined;
}

/** SQL without comments or quoted text, so neither can trigger a finding. */
export function stripSql(sql) {
  return sql
    .replace(/\/\*[\s\S]*?\*\//g, ' ')
    .replace(/--[^\n]*/g, ' ')
    .replace(/\$([A-Za-z_]*)\$[\s\S]*?\$\1\$/g, ' ')
    .replace(/'(?:[^']|'')*'/g, "''");
}

/** Names of the destructive statement kinds in `sql`. */
export function destructiveStatements(sql) {
  const code = stripSql(sql);
  return DESTRUCTIVE.filter(([, pattern]) => pattern.test(code)).map(([name]) => name);
}

/**
 * Compare the checkout's migrations with the versions the database has applied.
 *
 * Each pending file lists its statement kinds under `planned` when
 * `approvals` lists the file with these exact contents, else under `destructive`.
 */
export function migrationPlan(files, applied, read, approvals = {}) {
  const known = new Map();
  for (const file of files) {
    const version = migrationVersion(file);
    if (version !== undefined) known.set(version, file);
  }
  const done = new Set(applied.map(value => BigInt(value)));
  const ahead = [...done].filter(version => !known.has(version)).sort((a, b) => (a < b ? -1 : 1));
  const pending = [...known].filter(([version]) => !done.has(version)).sort(([a], [b]) => (a < b ? -1 : 1));
  return {
    ahead: ahead.map(String),
    pending: pending.map(([, file]) => {
      const sql = read(file);
      const kinds = destructiveStatements(sql);
      const reason = plannedRemoval(approvals, file, sql);
      return reason === undefined ? { file, destructive: kinds, planned: [] } : { file, destructive: [], planned: kinds, reason };
    }),
  };
}

/** The plan for the migration files in `directory`, with the installed planned removals by default. */
export function directoryPlan(directory, applied, approvals = readPlannedRemovals()) {
  return migrationPlan(readdirSync(directory), applied, file => readFileSync(join(directory, file), 'utf8'), approvals);
}

/** Whether applying `plan` would remove unplanned data or downgrade the schema. */
export function planBlocksUpdate(plan) {
  return plan.ahead.length > 0 || plan.pending.some(({ destructive }) => destructive.length > 0);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const [dir, approvalsFile] = process.argv.slice(2);
    if (!dir) throw new Error('Usage: migration-plan.mjs MIGRATIONS_DIR [PLANNED_REMOVALS_FILE] < applied-versions');
    const applied = readFileSync(0, 'utf8').split(/\s+/).filter(Boolean);
    const plan = directoryPlan(dir, applied, readPlannedRemovals(approvalsFile));
    for (const version of plan.ahead) console.log(`unknown applied migration ${version}`);
    for (const { file, destructive, planned, reason } of plan.pending) {
      console.log(`pending ${file}`);
      for (const kind of destructive) console.log(`destructive ${file}: ${kind}`);
      for (const kind of planned) console.log(`planned removal ${file}: ${kind} (${reason})`);
    }
    if (!plan.pending.length && !plan.ahead.length) console.log('database schema is up to date');
    if (plan.ahead.length) process.exitCode = 4;
    else if (plan.pending.some(({ destructive }) => destructive.length)) process.exitCode = 3;
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
