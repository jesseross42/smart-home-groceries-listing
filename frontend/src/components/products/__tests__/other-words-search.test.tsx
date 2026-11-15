import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { itemFixture } from "#/components/grocery/__tests__/fixtures";
import {
	productFixture,
	searchFixture,
	storeFixture,
} from "#/components/products/__tests__/fixtures";
import { ProductChoiceProvider } from "#/components/products/product-choice";
import { ProductResults } from "#/components/products/product-results";
import { api } from "#/lib/api";

afterEach(() => vi.restoreAllMocks());

describe("Search with other words", () => {
	it("searches the stores for the words, and chooses what it finds as looked up", async () => {
		const search = vi.spyOn(api.products, "forItem").mockResolvedValue(
			searchFixture([
				storeFixture({
					products: [productFixture({ product_id: "oat", name: "Oat Milk" })],
				}),
			]),
		);
		const onChoose = vi.fn().mockResolvedValue(undefined);
		render(
			<ProductChoiceProvider
				value={{ chosen: [], onChoose, onUnchoose: vi.fn() }}
			>
				<ProductResults item={itemFixture()} />
			</ProductChoiceProvider>,
		);
		await screen.findByRole("list", { name: "Products" });

		await userEvent.type(
			screen.getByLabelText("Search with other words"),
			" oat milk ",
		);
		await userEvent.click(screen.getByRole("button", { name: "Search" }));

		expect(search).toHaveBeenLastCalledWith(
			itemFixture().id,
			expect.anything(),
			"oat milk",
		);
		expect(
			await screen.findByText("Showing products for “oat milk”."),
		).toBeVisible();
		await userEvent.click(
			screen.getByRole("button", { name: "Choose Coles Oat Milk at Coles" }),
		);
		expect(onChoose).toHaveBeenCalledWith({
			store: "coles",
			product_id: "oat",
			looked_up: true,
		});

		await userEvent.click(
			screen.getByRole("button", { name: `Search for ${itemFixture().name}` }),
		);
		expect(search).toHaveBeenLastCalledWith(
			itemFixture().id,
			expect.anything(),
			undefined,
		);
	});
});
