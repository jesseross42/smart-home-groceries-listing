import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { DayPicker } from "#/components/order-options/day-picker";

describe("DayPicker", () => {
	it("marks the day the plan used and asks for another", async () => {
		const onChange = vi.fn();
		render(<DayPicker value="tuesday" storeFees={[]} onChange={onChange} />);

		expect(screen.getByRole("button", { name: "Tuesday" })).toHaveAttribute(
			"aria-pressed",
			"true",
		);
		await userEvent.click(screen.getByRole("button", { name: "Saturday" }));
		await userEvent.click(screen.getByRole("button", { name: "Cheapest day" }));

		expect(onChange.mock.calls).toEqual([["saturday"], ["any"]]);
		expect(screen.getByText(/Fees from Settings/)).toBeInTheDocument();
	});

	it("names the fees that came from the stores", () => {
		render(
			<DayPicker
				value={null}
				storeFees={[
					{
						store: "woolworths",
						store_name: "Woolworths",
						date: "2026-11-17",
						fee: "2",
					},
				]}
				onChange={vi.fn()}
			/>,
		);

		expect(
			screen.getByRole("button", { name: "Cheapest day" }),
		).toHaveAttribute("aria-pressed", "true");
		expect(screen.getByText(/Woolworths \$2\.00 on Tue/)).toBeInTheDocument();
	});
});
