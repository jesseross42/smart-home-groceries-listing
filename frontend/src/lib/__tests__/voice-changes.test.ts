import { describe, expect, it } from "vitest";

import { ApiError } from "#/lib/api";
import { describeChange, undoFailure, undoState } from "#/lib/voice-changes";

describe("describeChange", () => {
	it("says what a removed item had", () => {
		expect(
			describeChange({
				kind: "removed",
				quantity_before: 2,
				quantity_after: 0,
			}),
		).toBe("Removed (was ×2)");
	});

	it("shows a reduction as before and after", () => {
		expect(
			describeChange({
				kind: "reduced",
				quantity_before: 3,
				quantity_after: 1,
			}),
		).toBe("×3 → ×1");
	});
});

describe("undoState", () => {
	it("is null while the change can be undone", () => {
		expect(undoState({ undone_at: null, can_undo: true })).toBeNull();
	});

	it("says when it is undone, even if it is also old", () => {
		expect(
			undoState({ undone_at: "2026-10-02T04:40:16Z", can_undo: false }),
		).toBe("Undone");
	});

	it("says when it is too old", () => {
		expect(undoState({ undone_at: null, can_undo: false })).toBe(
			"Too old to undo",
		);
	});
});

describe("undoFailure", () => {
	it("gives the backend's reason for a refusal", () => {
		const refused = new ApiError(409, { detail: "the change is too old" });

		expect(undoFailure(refused)).toBe("Could not undo: the change is too old.");
	});

	it("asks to try again for anything else", () => {
		expect(undoFailure(new ApiError(500, "boom"))).toBe(
			"Could not undo that — try again.",
		);
		expect(undoFailure(new Error("offline"))).toBe(
			"Could not undo that — try again.",
		);
	});
});
