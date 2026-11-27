import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { MemberPricesForm } from "#/components/memberships/member-prices-form";
import type { StoreMembership } from "#/lib/api";

const woolworths: StoreMembership = {
	store: "woolworths",
	store_name: "Woolworths",
	program: "Everyday Rewards",
	member: false,
};

describe("MemberPricesForm", () => {
	it("shows each store's program and whether the household is a member", () => {
		render(
			<MemberPricesForm
				memberships={[{ ...woolworths, member: true }]}
				onChange={vi.fn()}
			/>,
		);

		expect(
			screen.getByRole("switch", {
				name: "Woolworths Everyday Rewards member",
			}),
		).toBeChecked();
	});

	it("reports the turned switch with its new value", async () => {
		const onChange = vi.fn();
		render(<MemberPricesForm memberships={[woolworths]} onChange={onChange} />);

		await userEvent.click(screen.getByRole("switch"));

		expect(onChange).toHaveBeenCalledWith({ ...woolworths, member: true });
	});
});
