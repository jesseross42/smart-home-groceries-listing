import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { productFixture } from "#/components/products/__tests__/fixtures";
import { ProductChoiceProvider } from "#/components/products/product-choice";
import { ProductLookup } from "#/components/products/product-lookup";
import { ApiError, api } from "#/lib/api";

const COLES_LINK =
	"https://www.coles.com.au/product/coles-full-cream-milk-3l-8150288";

function renderLookup(onChoose = vi.fn().mockResolvedValue(undefined)) {
	render(
		<ProductChoiceProvider
			value={{ chosen: [], onChoose, onUnchoose: vi.fn() }}
		>
			<ProductLookup />
		</ProductChoiceProvider>,
	);
	return onChoose;
}

async function lookUp(link: string) {
	await userEvent.type(screen.getByLabelText("Look up by link"), link);
	await userEvent.click(screen.getByRole("button", { name: "Look up" }));
}

afterEach(() => vi.restoreAllMocks());

describe("ProductLookup", () => {
	it("looks up a pasted link and chooses that product as looked up", async () => {
		const lookup = vi
			.spyOn(api.products, "lookup")
			.mockResolvedValue(productFixture({ product_id: "8150288" }));
		const onChoose = renderLookup();

		await lookUp(COLES_LINK);

		expect(lookup).toHaveBeenCalledWith("coles", "8150288");
		await userEvent.click(
			await screen.findByRole("button", {
				name: "Choose Coles Full Cream Milk at Coles",
			}),
		);
		expect(onChoose).toHaveBeenCalledWith({
			store: "coles",
			product_id: "8150288",
			looked_up: true,
		});
	});

	it("asks for a store link when the text is not one", async () => {
		const lookup = vi.spyOn(api.products, "lookup");
		renderLookup();

		await lookUp("https://example.com/milk");

		expect(screen.getByRole("alert")).toHaveTextContent(
			"Paste a product link from woolworths.com.au or coles.com.au.",
		);
		expect(lookup).not.toHaveBeenCalled();
	});

	it("says when the store does not sell it", async () => {
		vi.spyOn(api.products, "lookup").mockRejectedValue(
			new ApiError(404, { detail: "no" }),
		);
		renderLookup();

		await lookUp(COLES_LINK);

		expect(await screen.findByRole("alert")).toHaveTextContent(
			"Coles does not sell that product.",
		);
	});
});
