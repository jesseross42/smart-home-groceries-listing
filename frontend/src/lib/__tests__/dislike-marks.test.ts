import { afterEach, describe, expect, it, vi } from "vitest";

import { api } from "#/lib/api";
import { loadDislikeMarks, NO_DISLIKE_MARKS } from "#/lib/dislike-marks";

afterEach(() => vi.restoreAllMocks());

describe("loadDislikeMarks", () => {
	it("reads the dislikes and the overrides together", async () => {
		vi.spyOn(api.dislikes, "list").mockResolvedValue([]);
		const override = {
			grocery_item_id: "item-1",
			store: "coles" as const,
			product_id: "c-milk-3l",
			overridden_by: "me",
			overridden_at: "2026-10-02T10:00:00Z",
		};
		vi.spyOn(api.dislikes, "overrides").mockResolvedValue([override]);

		await expect(loadDislikeMarks()).resolves.toEqual({
			dislikes: [],
			overrides: [override],
		});
	});

	it("gives no marks when reading fails, so the page still loads", async () => {
		vi.spyOn(api.dislikes, "list").mockRejectedValue(new Error("offline"));
		vi.spyOn(api.dislikes, "overrides").mockResolvedValue([]);

		await expect(loadDislikeMarks()).resolves.toEqual(NO_DISLIKE_MARKS);
	});
});
