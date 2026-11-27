import { createFileRoute, useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import { MemberPricesForm } from "#/components/memberships/member-prices-form";
import { api, type StoreMembership } from "#/lib/api";
import { toastWithUndo } from "#/lib/undo-toast";

export const Route = createFileRoute("/settings/member-prices")({
	loader: () => api.storeMemberships.get(),
	component: MemberPricesPage,
});

/**
 * Settings › Member prices — the store rewards programs the household
 * belongs to. Members see and pay member prices; nobody else sees them.
 * Every change saves at once and shows a toast with Undo.
 */
function MemberPricesPage() {
	const { stores } = Route.useLoaderData();
	const router = useRouter();

	async function change(next: StoreMembership) {
		const previous = stores.find((m) => m.store === next.store);
		try {
			await api.storeMemberships.save([
				{ store: next.store, member: next.member },
			]);
		} catch {
			toast.error("Could not save. Try again.");
			return;
		}
		await router.invalidate();
		toastWithUndo("Member prices saved", {
			undo: () =>
				api.storeMemberships.save([
					{ store: next.store, member: previous?.member ?? false },
				]),
			failure: "Could not undo. Check the switch.",
			after: () => router.invalidate(),
		});
	}

	return (
		<div className="flex flex-col gap-4">
			<div className="flex flex-col gap-1">
				<h1 className="text-lg font-semibold">Member prices</h1>
				<p className="text-sm text-muted-foreground">
					Turn on a program you belong to. Its member prices are then used.
				</p>
			</div>
			<MemberPricesForm memberships={stores} onChange={change} />
		</div>
	);
}
