import { describe, expect, it } from "vitest";
import { parseProductLink } from "#/lib/product-link";

describe("parseProductLink", () => {
	it("reads a Woolworths product link", () => {
		expect(
			parseProductLink(
				"https://www.woolworths.com.au/shop/productdetails/888137/woolworths-full-cream-milk",
			),
		).toEqual({ store: "woolworths", product_id: "888137" });
	});

	it("reads a Coles product link", () => {
		expect(
			parseProductLink(
				"  https://www.coles.com.au/product/coles-full-cream-milk-3l-8150288?pid=x ",
			),
		).toEqual({ store: "coles", product_id: "8150288" });
	});

	it("names nothing for other links or words", () => {
		expect(parseProductLink("milk")).toBeNull();
		expect(
			parseProductLink("https://www.coles.com.au/browse/dairy-eggs-fridge"),
		).toBeNull();
		expect(
			parseProductLink("https://example.com/shop/productdetails/888137"),
		).toBeNull();
		expect(
			parseProductLink("https://notcoles.com.au/product/milk-8150288"),
		).toBeNull();
		expect(parseProductLink("javascript:alert(1)")).toBeNull();
	});
});
