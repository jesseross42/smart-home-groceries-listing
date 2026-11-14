import { useRouter } from "@tanstack/react-router";
import { useEffect } from "react";

import { connect, subscribe } from "#/lib/ws";

/** Events this close together cause one re-read, not one each. */
const SETTLE_MS = 200;

/**
 * Keeps a page in step with changes made elsewhere: another tab, another
 * household member, or Alexa. On `grocery_list_changed`, and whenever the
 * socket (re)connects, the route's loader runs again; the page's own changes
 * arrive the same way.
 */
export function useListSync(): void {
	const router = useRouter();

	useEffect(() => {
		let timer: ReturnType<typeof setTimeout> | undefined;
		connect();
		const unsubscribe = subscribe((event) => {
			// A snapshot opens every connection: after a reconnect, changes
			// made while the socket was down were never pushed, so read again.
			if (
				event.type !== "grocery_list_changed" &&
				event.type !== "counts_snapshot"
			)
				return;
			clearTimeout(timer);
			timer = setTimeout(() => void router.invalidate(), SETTLE_MS);
		});
		return () => {
			unsubscribe();
			clearTimeout(timer);
		};
	}, [router]);
}
