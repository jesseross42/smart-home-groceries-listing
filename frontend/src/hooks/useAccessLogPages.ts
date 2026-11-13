import { useState } from "react";
import { toast } from "sonner";

import { type AccessLogEntry, type AccessLogPage, api } from "#/lib/api";

/** The page's user and path filters; blank means "any". */
export interface AccessLogFilters {
	user: string;
	path: string;
}

const NO_FILTERS: AccessLogFilters = { user: "", path: "" };

/**
 * The `/settings/logs` page's rows: the first page as loaded, "Load older" to append
 * the next page, and the health-check switch and the filters, which each
 * start again from the newest rows. A failed load keeps what is shown, and
 * the switch and filters as they were, and says so.
 */
export function useAccessLogPages(initial: AccessLogPage) {
	const [entries, setEntries] = useState<AccessLogEntry[]>(initial.entries);
	const [nextBefore, setNextBefore] = useState(initial.next_before);
	const [showHealthChecks, setShowHealthChecks] = useState(false);
	const [filters, setFilters] = useState(NO_FILTERS);
	const [loading, setLoading] = useState(false);

	async function load(
		before: number | undefined,
		showHealth: boolean,
		{ user, path }: AccessLogFilters,
	) {
		setLoading(true);
		try {
			const page = await api.accessLogs.list({
				before,
				hideHealthChecks: !showHealth,
				user,
				path,
			});
			setEntries((prev) =>
				before === undefined ? page.entries : [...prev, ...page.entries],
			);
			setNextBefore(page.next_before);
			return true;
		} catch {
			toast.error("Could not load the access log — try again.");
			return false;
		} finally {
			setLoading(false);
		}
	}

	const loadOlder = () => {
		if (nextBefore !== null) void load(nextBefore, showHealthChecks, filters);
	};

	const changeHealthChecks = async (show: boolean) => {
		if (await load(undefined, show, filters)) setShowHealthChecks(show);
	};

	const changeFilters = async (next: AccessLogFilters) => {
		if (await load(undefined, showHealthChecks, next)) setFilters(next);
	};

	return {
		entries,
		hasOlder: nextBefore !== null,
		loading,
		showHealthChecks,
		filters,
		loadOlder,
		changeHealthChecks,
		changeFilters,
	};
}
