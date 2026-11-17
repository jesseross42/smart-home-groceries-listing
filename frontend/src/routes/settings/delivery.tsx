import { createFileRoute, useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import { DeliverySettingsForm } from "#/components/delivery/delivery-settings-form";
import { ApiError, api, type DeliverySettings } from "#/lib/api";
import { toastWithUndo } from "#/lib/undo-toast";

export const Route = createFileRoute("/settings/delivery")({
	loader: () => api.deliverySettings.get(),
	component: DeliveryPage,
});

/**
 * Settings › Delivery — each store's delivery fee rules and how the order
 * screen picks stores (spec: "Delivery constraints", "Optimisation modes").
 *
 * The app cannot read the stores' fees without your store login, so they are
 * typed here once. Saving shows a toast with Undo, which saves what was there
 * before. The loader stays the single source of truth.
 */
function DeliveryPage() {
	const settings = Route.useLoaderData();
	const router = useRouter();

	async function save(next: DeliverySettings, previous: DeliverySettings) {
		try {
			await api.deliverySettings.save(next);
		} catch (error) {
			toast.error(
				error instanceof ApiError &&
					error.message.includes("invalid_deadline_local_time")
					? "Choose a Sydney time outside the daylight-saving changeover."
					: "Could not save the delivery settings. Try again.",
			);
			throw error;
		}
		await router.invalidate();
		toastWithUndo("Delivery settings saved", {
			undo: () => api.deliverySettings.save(previous),
			failure: "Could not undo. Check the settings.",
			after: () => router.invalidate(),
		});
	}

	return (
		<div className="flex flex-col gap-4">
			<div className="flex flex-col gap-1">
				<h1 className="text-lg font-semibold">Delivery</h1>
				<p className="text-sm text-muted-foreground">
					Enter fallback fees and set when the household can take delivery.
				</p>
			</div>
			<DeliverySettingsForm
				key={JSON.stringify(settings)}
				settings={settings}
				onSave={(next) => save(next, settings)}
			/>
		</div>
	);
}
