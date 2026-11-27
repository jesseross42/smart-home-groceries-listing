import { useEffect } from "react";

import { announceHandoff } from "#/lib/desktop/announce-handoff";
import { filledNotice } from "#/lib/desktop/handoff-notice";
import { connect, subscribe } from "#/lib/ws";

/**
 * In the desktop app, notifies when the server pushes `trolley_filled`, so
 * the notice arrives even when the Send sheet is closed. Each handoff is
 * announced once. Does nothing in a browser.
 */
export function useTrolleyFilledNotices(): void {
	useEffect(() => {
		connect();
		return subscribe((event) => {
			if (event.type !== "trolley_filled") return;
			announceHandoff(
				event.handoff_id,
				filledNotice(event.store_name, event.added, event.not_added),
			);
		});
	}, []);
}
