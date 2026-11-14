import { useEffect } from "react";

import type { OrderOption, OrderOptions } from "#/lib/api";

/** Plans already applied this visit, so Undo is never fought. */
const applied = new Set<string>();

/** What an option buys where, as one comparable string. */
function signature(option: OrderOption): string {
	return option.picks
		.map((pick) => `${pick.grocery_item_id}:${pick.store}`)
		.sort()
		.join(",");
}

/**
 * Applies the recommended way to buy as soon as the Order page shows it
 * (Jesse, 2026-11-15): the cheapest of one store alone or both together,
 * delivery included. Each plan is applied once per visit, so pressing Undo
 * or **Use this** on another option sticks until the plan itself changes.
 */
export function useApplyBestOption(
	plan: OrderOptions,
	use: (option: OrderOption) => Promise<void>,
) {
	const best = plan.options.find((option) => option.recommended);
	const key = best ? signature(best) : "";
	// biome-ignore lint/correctness/useExhaustiveDependencies: the plan's signature is the trigger.
	useEffect(() => {
		if (!best || best.is_current || applied.has(key)) return;
		applied.add(key);
		void use(best);
	}, [key, best?.is_current]);
}

/** Forgets applied plans; for tests. */
export function forgetAppliedPlans() {
	applied.clear();
}
