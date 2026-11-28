/**
 * Planned data removals: the data-removing migrations a production update may apply.
 *
 * `planned-removals.json` maps a migration file name to the SHA-256 of its
 * exact contents and the reason the removal is safe. The list is control
 * code: it is installed from a reviewed checkout next to this module and is
 * never read from a release bundle, so a release cannot approve itself.
 */
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

/** The list installed next to this module. */
const INSTALLED_LIST = new URL('./planned-removals.json', import.meta.url);

const isEntry = value => value !== null && typeof value === 'object'
  && typeof value.sha256 === 'string' && /^[a-f0-9]{64}$/.test(value.sha256)
  && typeof value.reason === 'string' && value.reason.trim() !== '';

/** SHA-256 of a migration's contents, as `sha256sum` prints it. */
export const contentHash = sql => createHash('sha256').update(sql).digest('hex');

/** Parse and validate the list's JSON text. A malformed list is an error, never an empty list. */
export function parsePlannedRemovals(text) {
  let list;
  try { list = JSON.parse(text); } catch { throw new Error('Planned removals: not valid JSON'); }
  if (list === null || typeof list !== 'object' || Array.isArray(list)) {
    throw new Error('Planned removals: expected an object keyed by migration file name');
  }
  for (const [file, entry] of Object.entries(list)) {
    if (!/^\d+_[A-Za-z0-9_.-]+\.sql$/.test(file) || !isEntry(entry)) {
      throw new Error(`Planned removals: ${JSON.stringify(file)} needs a migration file name, a sha256 and a reason`);
    }
  }
  return list;
}

/** Read the list from `path`, by default the one installed next to this module. */
export function readPlannedRemovals(path = INSTALLED_LIST) {
  return parsePlannedRemovals(readFileSync(path, 'utf8'));
}

/** The approval reason when `file` is listed with exactly the contents `sql`, else `undefined`. */
export function plannedRemoval(approvals, file, sql) {
  if (!Object.hasOwn(approvals, file)) return undefined;
  const { sha256, reason } = approvals[file];
  return contentHash(sql) === sha256 ? reason : undefined;
}
