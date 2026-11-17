import { Link } from "@tanstack/react-router";

import { deadlineLabel } from "#/components/order-options/window-status";
import { FillTrolleyBookmark } from "#/components/trolley/fill-trolley-bookmark";
import { STORE_TABS } from "#/lib/store-tab/store-tabs";

/** Shows the active deadline and the store bookmarks that refresh availability. */
export function RefreshDeliveryWindows({ needBy }: { needBy: string }) {
	return (
		<div className="flex flex-col gap-2 text-sm">
			<p>
				Need by <strong>{deadlineLabel(needBy)}</strong> Sydney time.{" "}
				<Link to="/settings/delivery" className="underline">
					Edit delivery settings
				</Link>
			</p>
			<details className="rounded-md border border-border p-3">
				<summary className="w-fit cursor-pointer font-medium">
					Refresh delivery windows
				</summary>
				<p className="mt-2 text-muted-foreground">
					Drag each bookmark to your bookmarks bar. Open the store, sign in,
					select delivery to your address and run the bookmark. Return here and
					select Check prices again. Window reports expire after 15 minutes.
				</p>
				<div className="mt-2 flex flex-wrap gap-2">
					<FillTrolleyBookmark tab={STORE_TABS.woolworths} refreshWindows />
					<FillTrolleyBookmark tab={STORE_TABS.coles} refreshWindows />
				</div>
			</details>
			<p className="text-xs text-muted-foreground">
				Confirm the matching delivery window on the store’s website before
				paying.
			</p>
		</div>
	);
}
