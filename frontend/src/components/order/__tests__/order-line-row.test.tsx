import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { dislikeFixture } from "#/components/dislikes/__tests__/fixtures";
import { DislikeMarksProvider } from "#/components/dislikes/dislike-mark";
import { lineFixture } from "#/components/order/__tests__/fixtures";
import { OrderLineRow } from "#/components/order/order-line-row";

function renderLine(overrides = {}) {
	render(
		<ul>
			<OrderLineRow line={lineFixture(overrides)} store="coles" />
		</ul>,
	);
	return screen.getByRole("listitem", { name: "milk" });
}

describe("OrderLineRow", () => {
	it("shows the item, its quantity, the product and today's total", () => {
		const row = renderLine();

		expect(row).toHaveTextContent("milk");
		expect(row).toHaveTextContent("× 2");
		expect(row).toHaveTextContent("Coles Full Cream Milk");
		expect(row).toHaveTextContent("3L");
		expect(row).toHaveTextContent("$9.90");
		expect(row).toHaveTextContent("$4.95 each");
	});

	it("links the product to its page at the store", () => {
		renderLine();

		const link = screen.getByRole("link", { name: /Full Cream Milk/ });
		expect(link).toHaveAttribute(
			"href",
			"https://www.coles.com.au/product/coles-full-cream-milk-3l-8150288",
		);
		expect(link).toHaveAttribute("target", "_blank");
	});

	it("flags a price that went up since it was chosen", () => {
		const row = renderLine({ price_change: "up", chosen_price: "4.50" });

		expect(row).toHaveTextContent("Up from $4.50");
	});

	it("says when a deal lowers the total", () => {
		const row = renderLine({ deal_applied: true });

		expect(row).toHaveTextContent("Deal applied");
	});

	it("shows the problem, and no price, when the line cannot be bought", () => {
		const row = renderLine({
			status: "not_offered",
			price: null,
			total_price: null,
			price_change: null,
			problem:
				"Coles no longer offers this product for milk — choose another on the list.",
		});

		expect(row).toHaveTextContent("No price");
		expect(row).not.toHaveTextContent("$9.90");
		expect(row).toHaveTextContent("no longer offers this product");
	});

	it("shows the chosen product's picture", () => {
		const row = renderLine();

		expect(
			within(row).getByRole("img", { name: "Coles Full Cream Milk" }),
		).toHaveAttribute(
			"src",
			"https://productimages.coles.com.au/productimages/8/8150288.jpg",
		);
	});
});

describe("OrderLineRow dislikes", () => {
	it("warns when somebody dislikes the line's product", () => {
		render(
			<DislikeMarksProvider
				value={{ dislikes: [dislikeFixture()], overrides: [] }}
			>
				<ul>
					<OrderLineRow line={lineFixture()} store="coles" />
				</ul>
			</DislikeMarksProvider>,
		);

		expect(
			within(screen.getByRole("listitem", { name: "milk" })).getByRole(
				"status",
			),
		).toHaveTextContent("Phu disliked this item previously.");
	});

	it("does not warn for the same product id at another store", () => {
		render(
			<DislikeMarksProvider
				value={{ dislikes: [dislikeFixture()], overrides: [] }}
			>
				<ul>
					<OrderLineRow line={lineFixture()} store="woolworths" />
				</ul>
			</DislikeMarksProvider>,
		);

		expect(screen.queryByTestId("dislike-mark")).not.toBeInTheDocument();
	});
});
