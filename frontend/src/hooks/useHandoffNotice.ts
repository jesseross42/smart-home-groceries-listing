import { useEffect } from "react";

import type { TrolleyHandoff } from "#/lib/api/trolley-handoffs";
import { announceHandoff } from "#/lib/desktop/announce-handoff";
import { handoffNotice } from "#/lib/desktop/handoff-notice";

/**
 * In the desktop app, sends one notification when `handoff` is filled — so a
 * person who looked away from the sheet hears that the trolley is ready.
 * Each handoff is announced once, also when the server pushed it first.
 */
export function useHandoffNotice(handoff: TrolleyHandoff | null): void {
	useEffect(() => {
		if (!handoff) return;
		const notice = handoffNotice(handoff);
		if (notice) announceHandoff(handoff.id, notice);
	}, [handoff]);
}
