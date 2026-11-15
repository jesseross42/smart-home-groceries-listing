import { renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
	forgetAppliedPlans,
	useApplyBestOption,
} from "#/hooks/useApplyBestOption";
import type { OrderOption, OrderOptions } from "#/lib/api";

function option(overrides: Partial<OrderOption>): OrderOption {
	return {
		kind: "split",
		label: "Split between stores",
		recommended: false,
		is_current: false,
		stores: [],
		missing: [],
		disliked: [],
		items_total: "10",
		delivery_total: "0",
		total: "10",
		complete: true,
		fees_known: true,
		meets_minimums: true,
		within_delivery_cap: true,
		picks: [{ grocery_item_id: "i1", store: "woolworths" }],
		...overrides,
	};
}

function plan(options: OrderOption[]): OrderOptions {
	return { options } as unknown as OrderOptions;
}

afterEach(() => forgetAppliedPlans());

describe("useApplyBestOption", () => {
	it("applies the recommended option once", () => {
		const use = vi.fn().mockResolvedValue(undefined);
		const best = option({ recommended: true });
		const { rerender } = renderHook(() =>
			useApplyBestOption(plan([best, option({ kind: "coles" })]), use),
		);
		rerender();

		expect(use).toHaveBeenCalledTimes(1);
		expect(use).toHaveBeenCalledWith(best);
	});

	it("leaves the order alone when the best is already bought or none is recommended", () => {
		const use = vi.fn().mockResolvedValue(undefined);
		renderHook(() =>
			useApplyBestOption(
				plan([option({ recommended: true, is_current: true })]),
				use,
			),
		);
		renderHook(() => useApplyBestOption(plan([option({})]), use));

		expect(use).not.toHaveBeenCalled();
	});
});
