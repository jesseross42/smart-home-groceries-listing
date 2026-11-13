import { useEffect } from "react";

import { announceFilledOnce, filledNotice } from "#/lib/desktop/handoff-notice";
import { connect, subscribe } from "#/lib/ws";

/**
 * In the desktop app, notifies when the server says a trolley was filled,
 * so the notice arrives even when the handoff's sheet is closed. Each
 * handoff is announced once, however often the event arrives.
 *
 * Events are not replayed after a reconnect: a fill reported while the
 * socket was down is announced by the sheet (`useHandoffNotice`) if it is
 * open, and otherwise not at all.
 */
export function useTrolleyFilledNotice(): void {
	useEffect(() => {
		connect();
		return subscribe((event) => {
			if (event.type !== "trolley_filled") return;
			announceFilledOnce(
				event.handoff_id,
				filledNotice(event.store_name, event.added, event.failed),
			);
		});
	}, []);
}
