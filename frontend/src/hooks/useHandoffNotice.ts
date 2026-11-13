import { useEffect } from "react";

import type { TrolleyHandoff } from "#/lib/api/trolley-handoffs";
import {
	announceFilledOnce,
	handoffNotice,
} from "#/lib/desktop/handoff-notice";

/**
 * In the desktop app, sends one notification when `handoff` is filled — so a
 * person who looked away from the sheet hears that the trolley is ready.
 * Each handoff is announced once, even when the server's push
 * (`useTrolleyFilledNotice`) reports it too.
 */
export function useHandoffNotice(handoff: TrolleyHandoff | null): void {
	useEffect(() => {
		if (!handoff) return;
		const notice = handoffNotice(handoff);
		if (notice) announceFilledOnce(handoff.id, notice);
	}, [handoff]);
}
