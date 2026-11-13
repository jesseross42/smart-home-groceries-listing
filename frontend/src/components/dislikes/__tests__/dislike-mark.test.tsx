import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { dislikeFixture } from "#/components/dislikes/__tests__/fixtures";
import {
	DislikeMark,
	DislikeMarksProvider,
} from "#/components/dislikes/dislike-mark";
import type { DislikeMarks } from "#/lib/dislike-marks";

const ITEM = "item-1";

function renderMark(marks: DislikeMarks | null, productId = "c-milk-3l") {
	const mark = (
		<DislikeMark itemId={ITEM} store="coles" productId={productId} />
	);
	return render(
		marks ? (
			<DislikeMarksProvider value={marks}>{mark}</DislikeMarksProvider>
		) : (
			mark
		),
	);
}

describe("DislikeMark", () => {
	it("says who dislikes the product", () => {
		renderMark({ dislikes: [dislikeFixture()], overrides: [] });

		expect(screen.getByRole("status")).toHaveTextContent(
			"Phu disliked this item previously.",
		);
	});

	it("says when the dislike is set aside for this item", () => {
		renderMark({
			dislikes: [dislikeFixture()],
			overrides: [
				{
					grocery_item_id: ITEM,
					store: "coles",
					product_id: "c-milk-3l",
					overridden_by: "me",
					overridden_at: "2026-10-02T10:00:00Z",
				},
			],
		});

		expect(screen.getByTestId("dislike-mark")).toHaveTextContent(
			"OK to buy it this time.",
		);
	});

	it("shows nothing for a product nobody dislikes", () => {
		renderMark({ dislikes: [dislikeFixture()], overrides: [] }, "c-bread");

		expect(screen.queryByTestId("dislike-mark")).not.toBeInTheDocument();
	});

	it("shows nothing outside a provider", () => {
		renderMark(null);

		expect(screen.queryByTestId("dislike-mark")).not.toBeInTheDocument();
	});
});
